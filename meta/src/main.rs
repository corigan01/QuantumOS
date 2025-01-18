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

async fn build() -> Result<PathBuf> {
    let (artifacts, disk) = tokio::join!(build_project(), DiskImgBaker::new());

    let artifacts = artifacts.expect("Failed to build artifacts!");
    let mut disk = disk?;

    disk.write_bootsector(&artifacts.bootsector).await?;
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
    disk.finish_and_write().await
}

fn run_qemu(
    disk_target_path: &Path,
    enable_kvm: bool,
    enable_no_graphic: bool,
    log_interrupts: bool,
    slow_emu: bool,
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

    Command::new("qemu-system-x86_64")
        .args(kvm)
        .args(no_graphic)
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
            build().await?;
        }
        cmdline::TaskOption::Run => {
            if !args.use_bochs {
                run_qemu(
                    &build().await?,
                    args.enable_kvm,
                    args.no_graphic,
                    args.log_interrupts,
                    args.slow_emulator,
                )?;
            } else {
                run_bochs(&build().await?).await?;
            }
        }
        cmdline::TaskOption::BuildDisk => {
            run_mk_image(&build().await?).await?;
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
    }

    Ok(())
}
