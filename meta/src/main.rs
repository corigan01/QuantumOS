#![feature(async_fn_traits)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    artifacts::build_project,
    disk::{create_bootloader_dir, DiskImgBaker},
};

mod artifacts;
mod cmdline;
mod disk;

struct QuickBootImages {
    // Address, Pat
    kernel_img: (usize, usize, PathBuf),
    initfs_img: (usize, usize, PathBuf),
    loader32: (usize, usize, PathBuf),
    loader64: (usize, usize, PathBuf),
    data_ptr: usize,
}

struct BuildResult {
    disk_img: PathBuf,
    quick_boot: Option<QuickBootImages>,
}

async fn build(multiboot_mode: bool, emit_asm: Option<String>) -> Result<BuildResult> {
    let (artifacts, disk) =
        tokio::join!(build_project(multiboot_mode, emit_asm), DiskImgBaker::new());

    let artifacts = artifacts.expect("Failed to build artifacts!");
    let mut disk = disk?;

    disk.write_bootsector(&artifacts.bootsector).await?;

    let quick_boot = if !multiboot_mode {
        disk.write_stage16(&artifacts.stage_16).await?;

        let bootloader_dir_path = create_bootloader_dir(
            "fatfs",
            [
                (
                    artifacts.bootsector,
                    PathBuf::from("bootloader/bootsector.bin"),
                ),
                (artifacts.stage_16, PathBuf::from("bootloader/stage_16.bin")),
                (artifacts.boot_cfg, PathBuf::from("bootloader/qconfig.cfg")),
                (artifacts.stage_32, PathBuf::from("bootloader/stage_32.bin")),
                (artifacts.stage_64, PathBuf::from("bootloader/stage_64.bin")),
                (artifacts.kernel, PathBuf::from("kernel.elf")),
                (artifacts.initfs, PathBuf::from("initfs")),
            ]
            .into_iter(),
        )
        .await?;

        disk.dir_to_fat(&bootloader_dir_path).await?;

        // We built the normal bootloader, so no need to emit fast boot binaries
        None
    } else {
        let data_ptr = 0x00100000;

        let loader32_ptr = 0x00200000;
        let loader32_size = 0x00100000;

        let loader64_ptr = 0x00400000;
        let loader64_size = 0x00100000;

        let kernel_ptr = 0x00600000;
        let kernel_size = artifacts.kernel_len;

        let initfs_ptr = kernel_ptr + kernel_size;
        let initfs_len = artifacts.initfs_len;

        // `build_project` will emit different binaries depending on how its configured
        Some(QuickBootImages {
            kernel_img: (kernel_ptr, kernel_size, artifacts.kernel),
            initfs_img: (initfs_ptr, initfs_len, artifacts.initfs),
            loader32: (loader32_ptr, loader32_size, artifacts.stage_32),
            loader64: (loader64_ptr, loader64_size, artifacts.stage_64),
            data_ptr,
        })
    };

    let disk_img = disk.finish_and_write().await?;

    Ok(BuildResult {
        disk_img,
        quick_boot,
    })
}

fn run_qemu(
    disk_target_path: &Path,
    enable_kvm: bool,
    enable_no_graphic: bool,
    log_interrupts: bool,
    slow_emu: bool,
    quick_boot: Option<QuickBootImages>,
) -> Result<()> {
    let kvm: &[&str] = if enable_kvm {
        &["--enable-kvm", "--cpu", "host"]
    } else {
        &[]
    };
    let no_graphic: &[&str] = if enable_no_graphic {
        &["-nographic", "-serial", "mon:stdio"]
    } else {
        &["-serial", "stdio"]
    };
    let log_interrupts: &[&str] = if log_interrupts {
        &["-d", "int"]
    } else {
        &["-d", "cpu_reset"]
    };
    let slow_emulator: &[&str] = if slow_emu {
        &["-icount", "10,align=on"]
    } else {
        &[]
    };
    let fast_boot: &[&str] = if let Some(quick_boot) = quick_boot {
        &[
            // Stage32
            "-kernel",
            &format!("{}", quick_boot.loader32.2.to_string_lossy()),
            // Stage64
            "-device",
            &format!(
                "loader,addr={},file={},force-raw=on",
                quick_boot.loader64.0,
                quick_boot.loader64.2.to_string_lossy()
            ),
            // Kernel
            "-device",
            &format!(
                "loader,addr={},file={},force-raw=on",
                quick_boot.kernel_img.0,
                quick_boot.kernel_img.2.to_string_lossy()
            ),
            // initfs
            "-device",
            &format!(
                "loader,addr={},file={},force-raw=on",
                quick_boot.initfs_img.0,
                quick_boot.initfs_img.2.to_string_lossy()
            ),
            // Write options into memory (Stage32_ptr)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 0),
                quick_boot.loader32.0
            ),
            // Write options into memory (Stage32_len)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 1),
                quick_boot.loader32.1
            ),
            // Write options into memory (Stage64_ptr)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 2),
                quick_boot.loader64.0
            ),
            // Write options into memory (Stage64_len)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 3),
                quick_boot.loader64.1
            ),
            // Write options into memory (Kernel_ptr)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 4),
                quick_boot.kernel_img.0
            ),
            // Write options into memory (Kernel_len)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 5),
                quick_boot.kernel_img.1
            ),
            // Write options into memory (initfs_ptr)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 6),
                quick_boot.initfs_img.0
            ),
            // Write options into memory (initfs_len)
            "-device",
            &format!(
                "loader,addr={},data={:#016x},data-len=8",
                quick_boot.data_ptr + (8 * 7),
                quick_boot.initfs_img.1
            ),
        ]
    } else {
        &[]
    };

    Command::new("qemu-system-x86_64")
        .args(kvm)
        .args(no_graphic)
        .args(fast_boot)
        .arg("--name")
        .arg("Quantum OS")
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
        .arg("--no-reboot")
        .args(log_interrupts)
        .arg("-m")
        .arg("256M")
        .arg("-k")
        .arg("en-us")
        .arg("-nic")
        .arg("none")
        .args(slow_emulator)
        .arg("-drive")
        .arg(format!(
            "format=raw,file={}",
            disk_target_path.to_str().unwrap()
        ))
        .stdout(std::process::Stdio::inherit())
        .status()
        .context(anyhow!("Could not start qemu-system-x86_64!"))?
        .success()
        .then_some(())
        .ok_or(anyhow!("QEMU Failed"))?;

    Ok(())
}

async fn run_bochs(img_file: &Path) -> Result<()> {
    Command::new("bochs")
        .arg("-n")
        .arg("-q")
        .arg("boot:disk")
        .arg("megs: 512")
        .arg("ata0: enabled=1")
        .arg(format!(
            "ata0-master: type=disk, path={}, mode=flat, translation=auto",
            img_file.to_str().unwrap()
        ))
        .arg("cpuid: x86_64=1, level=6")
        .arg("display_library: sdl2")
        .arg("com1: enabled=1, mode=file, dev=./log.log")
        .stdout(std::process::Stdio::inherit())
        .status()
        .context(anyhow!("Could not start bochs!"))?
        .success()
        .then_some(())
        .ok_or(anyhow!("bochs Failed"))?;

    Ok(())
}

async fn run_mk_image(img_file: &Path) -> Result<()> {
    Command::new("qemu-img")
        .arg("convert")
        .arg("-f")
        .arg("raw")
        .arg("-O")
        .arg("qcow2")
        .arg(img_file)
        .arg("quantum_os.qcow2")
        .stdout(std::process::Stdio::inherit())
        .status()
        .context(anyhow!("Could not start qemu-img!"))?
        .success()
        .then_some(())
        .ok_or(anyhow!("qemu-img Failed"))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cmdline::CommandLine::parse();

    match args.option.unwrap_or(cmdline::TaskOption::Run) {
        cmdline::TaskOption::Build => {
            build(false, None).await?;
        }
        cmdline::TaskOption::Run => {
            if !args.use_bochs {
                run_qemu(
                    &build(false, None).await?.disk_img,
                    args.enable_kvm,
                    args.no_graphic,
                    args.log_interrupts,
                    args.slow_emulator,
                    None,
                )?;
            } else {
                run_bochs(&build(false, None).await?.disk_img).await?;
            }
        }
        cmdline::TaskOption::RunQuick => {
            if args.use_bochs {
                panic!("Bochs is not supported with quick-load mode! Please use QEMU, or switch to using default bootloader mode!");
            }

            let BuildResult {
                disk_img,
                quick_boot: Some(quick_boot),
            } = build(true, None).await?
            else {
                panic!("Build didn't return expected results!");
            };

            run_qemu(
                &disk_img,
                args.enable_kvm,
                args.no_graphic,
                args.log_interrupts,
                args.slow_emulator,
                Some(quick_boot),
            )?;
        }
        cmdline::TaskOption::BuildDisk => {
            run_mk_image(&build(false, None).await?.disk_img).await?;
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
        cmdline::TaskOption::EmitAsm { file } => {
            build(false, Some(file)).await?;
        }
    }

    Ok(())
}
