#![feature(async_closure)]
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
                artifacts.bootsector.as_path(),
                Path::new("bootloader/bootsector.bin"),
            ),
            (
                &artifacts.stage_16.as_path(),
                Path::new("bootloader/stage_16.bin"),
            ),
            (
                &artifacts.stage_32.as_path(),
                Path::new("bootloader/stage_32.bin"),
            ),
            (&artifacts.kernel.as_path(), Path::new("kernel.elf")),
            (
                &artifacts.boot_cfg.as_path(),
                Path::new("bootloader/qconfig.cfg"),
            ),
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
) -> Result<()> {
    let kvm: &[&str] = if enable_kvm { &["--enable-kvm"] } else { &[] };
    let no_graphic: &[&str] = if enable_no_graphic {
        &["-nographic", "-serial", "mon:stdio"]
    } else {
        &[]
    };
    let log_interrupts: &[&str] = if log_interrupts { &["-d", "int"] } else { &[] };

    Command::new("qemu-system-x86_64")
        .args(kvm)
        .args(no_graphic)
        .args(log_interrupts)
        .arg("--name")
        .arg("Quantum OS")
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
        .arg("-d")
        .arg("int")
        .arg("-d")
        .arg("cpu_reset")
        .arg("--no-reboot")
        .arg("-m")
        .arg("256M")
        .arg("-k")
        .arg("en-us")
        .arg("-nic")
        .arg("none")
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = cmdline::CommandLine::parse();

    match args.option.unwrap_or(cmdline::TaskOption::Run) {
        cmdline::TaskOption::Build => {
            build().await?;
        }
        cmdline::TaskOption::Run => {
            run_qemu(
                &build().await?,
                args.enable_kvm,
                args.no_graphic,
                args.log_interrupts,
            )?;
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
    }

    Ok(())
}
