use anyhow::{Context, Error, Result};
use async_process::{Command, Stdio};
use futures::future;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug)]
pub struct Artifacts {
    pub bootsector: PathBuf,
    pub stage_16: PathBuf,
    pub stage_32: PathBuf,
    pub stage_64: PathBuf,

    pub kernel: PathBuf,
    pub boot_cfg: PathBuf,
}

#[allow(unused)]
enum ArchSelect {
    /// # Intel 368 (16bit mode)
    I386,
    /// # Intel 686 (32bit mode)
    I686,
    /// # Intel IA-32A (64bit mode)
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

    Command::new("cargo")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env("CARGO_TERM_PROGRESS_WHEN", "never")
        .args([
            "build",
            "--package",
            package,
            "--profile",
            compile_mode,
            "--target",
            arch.to_string().as_str(),
            "--artifact-dir",
            "./target/bin",
            "-Zbuild-std=core",
            "-Zbuild-std-features=compiler-builtins-mem",
            "-Zunstable-options",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .await?
        .success()
        .then_some(())
        .ok_or(Error::msg("Failed to run Cargo"))?;

    Ok(PathBuf::from("./target")
        .join("bin/")
        .join(package)
        .canonicalize()?)
}

async fn convert_bin(path: &Path, arch: ArchSelect) -> Result<PathBuf> {
    let arch = match arch {
        ArchSelect::I386 => "elf32-i386",
        ArchSelect::I686 | ArchSelect::X64 => "elf64-x86-64",
    };

    let bin_path = path.with_extension("bin");
    fs::copy(path, &bin_path).context("Failed to duplicate ELF output file")?;

    Command::new("objcopy")
        .args([
            "-I",
            arch,
            "-O",
            "binary",
            &bin_path.as_path().to_str().unwrap(),
        ])
        .status()
        .await
        .context("Failed to convert ELF file to BIN")?
        .success()
        .then_some(())
        .ok_or(Error::msg("Failed to run objcopy"))?;

    Ok(bin_path)
}

async fn build_bootloader_config() -> Result<PathBuf> {
    let target_location = PathBuf::from("./target/qconfig.cfg");

    let mut file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&target_location)
        .await?;

    file.write_all(
        br#"bootloader32=/bootloader/stage_32.bin
bootloader64=/bootloader/stage_64.bin
kernel=/kernel.elf
vbe-mode=1280x720
"#,
    )
    .await?;

    Ok(target_location)
}

pub async fn build_project() -> Result<Artifacts> {
    let (stage_bootsector, stage_16bit, stage_32bit, stage_64bit, kernel, boot_cfg) = tokio::try_join!(
        cargo_helper(
            Some("stage-bootsector"),
            "stage-bootsector",
            ArchSelect::I386,
        ),
        cargo_helper(Some("stage-16bit"), "stage-16bit", ArchSelect::I386),
        cargo_helper(Some("stage-32bit"), "stage-32bit", ArchSelect::I686),
        cargo_helper(Some("stage-64bit"), "stage-64bit", ArchSelect::X64),
        cargo_helper(None, "kernel", ArchSelect::X64),
        build_bootloader_config(),
    )?;

    let (bootsector, stage_16, stage_32, stage_64) = future::try_join4(
        convert_bin(&stage_bootsector, ArchSelect::I386),
        convert_bin(&stage_16bit, ArchSelect::I386),
        convert_bin(&stage_32bit, ArchSelect::I686),
        convert_bin(&stage_64bit, ArchSelect::X64),
    )
    .await?;

    Ok(Artifacts {
        bootsector,
        stage_16,
        stage_32,
        stage_64,
        kernel,
        boot_cfg,
    })
}
