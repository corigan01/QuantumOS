use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use async_process::Command;

#[derive(Clone, Debug)]
pub struct BootloaderArtifacts {
    pub bootsector: PathBuf,
    pub stage_16: PathBuf,
    // stage_32: PathBuf,
    // stage_64: PathBuf
}

pub async fn build_bootloader(project_root: &Path, release: bool) -> Result<BootloaderArtifacts> {
    Command::new("cargo")
        .env_clear()
        .env("PATH", env::var("PATH")?)
        .current_dir(project_root.join("bootloader"))
        .args([
            "build",
            "--profile",
            if release { "release" } else { "dev" },
        ])
        .status()
        .await?
        .success()
        .then_some(())
        .context("Could not build Quantum-OS's Bootloader!")?;

    Ok(BootloaderArtifacts {
        bootsector: project_root
            .join("bootloader/target/bin/stage-bootsector.bin")
            .canonicalize()
            .unwrap(),
        stage_16: project_root
            .join("bootloader/target/bin/stage-16bit.bin")
            .canonicalize()?,
    })
}
