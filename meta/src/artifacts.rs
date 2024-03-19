use anyhow::{Context, Error, Result};
use async_process::{Command, Stdio};
use futures::{future, Future};
use std::env;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct BootloaderArtifacts {
    pub bootsector: PathBuf,
    pub stage_16: PathBuf,
    // stage_32: PathBuf,
    // stage_64: PathBuf
}

#[allow(unused)]
enum ArchSelect {
    I386,
    I686,
    X64,
}

impl Display for ArchSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_dir = Path::new("./bootloader/");
        match self {
            Self::I386 => f.write_fmt(format_args!(
                "{}",
                current_dir
                    .join("linkerscripts/i386-quantum_loader.json")
                    .to_string_lossy(),
            )),
            Self::I686 => f.write_fmt(format_args!(
                "{}",
                current_dir
                    .join("linkerscripts/i686-quantum_loader.json")
                    .to_string_lossy(),
            )),
            Self::X64 => f.write_fmt(format_args!(
                "{}",
                current_dir
                    .join("linkerscripts/x86-64-quantum_loader.json")
                    .to_string_lossy(),
            )),
        }
    }
}

async fn cargo_helper(profile: Option<&str>, package: &str, arch: ArchSelect) -> Result<PathBuf> {
    let compile_mode = profile.unwrap_or("release");
    println!("cargo:rerun-if-changed={}", package);

    Command::new("cargo")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .args([
            "install",
            "--path",
            package,
            "--profile",
            compile_mode,
            "--target",
            arch.to_string().as_str(),
            "--root",
            "./target",
            "-Zbuild-std=core",
            "-Zbuild-std-features=compiler-builtins-mem",
        ])
        .stdout(Stdio::inherit())
        .status()
        .await?
        .success()
        .then_some(())
        .ok_or(Error::msg("Failed to run Cargo"))?;

    Ok(PathBuf::from("./target").join("bin").join(package))
}

async fn convert_bin(path: &Path, arch: ArchSelect) -> Result<PathBuf> {
    let arch = match arch {
        ArchSelect::I386 => "elf32-i386",
        _ => todo!("Add more objcopy arches"),
    };

    let bin_path = path.join(".bin");
    fs::copy(path, &bin_path)?;

    Command::new("objcopy")
        .args([
            "-I",
            arch,
            "-O",
            "binary",
            &bin_path.as_path().to_str().unwrap(),
        ])
        .status()
        .await?
        .success()
        .then_some(())
        .ok_or(Error::msg("Failed to run objcopy"))?;

    Ok(bin_path)
}

pub async fn build_project(project_root: &Path, release: bool) -> Result<BootloaderArtifacts> {
    let (stage_bootsector, stage_16bit, kernel) = future::try_join3(
        cargo_helper(
            Some("stage-bootsector"),
            "./bootloader/stage-bootsector",
            ArchSelect::I386,
        ),
        cargo_helper(
            Some("stage-16bit"),
            "./bootloader/stage-16bit",
            ArchSelect::I386,
        ),
        cargo_helper(None, "./kernel", ArchSelect::X64),
    )
    .await?;

    todo!()
}
