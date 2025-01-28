use anyhow::{Context, Error, Result};
use async_process::{Command, Stdio};
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
    pub kernel_len: usize,
    pub boot_cfg: PathBuf,

    pub initfs: PathBuf,
    pub initfs_len: usize,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
enum ArchSelect {
    /// # Intel 368 (16bit mode)
    I386,
    /// # Intel 686 (32bit mode)
    I686,
    /// # Intel IA-32e (64bit mode)
    X64,
    /// # Intel IA-32e (64bit mode)
    Kernel,
    /// # Intel IA-32e (64bit mode) -- Userspace Mode
    UserSpace,
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
            Self::Kernel => f.write_fmt(format_args!(
                "{}",
                current_dir
                    .join("../kernel/x86-64-quantum_kernel.json")
                    .to_string_lossy(),
            )),
            Self::UserSpace => {
                f.write_fmt(format_args!("{}", "./user/x86_64-unknown-quantum.json",))
            }
        }
    }
}

async fn cargo_helper(
    profile: Option<&str>,
    package: &str,
    arch: ArchSelect,
    feature_flags: Option<&str>,
    should_emit_asm: bool,
) -> Result<PathBuf> {
    let compile_mode = profile.unwrap_or("release");

    let build_std_options: &[&str] = if package == "kernel" {
        &["-Zbuild-std=core,alloc"]
    } else {
        &["-Zbuild-std=core"]
    };

    let feature_flags: &[&str] = if let Some(feature_flags) = feature_flags {
        &["--features", feature_flags]
    } else {
        &[]
    };

    let arch_string = arch.to_string();
    let (pre_build_command, post_build_command): (&[&str], &[&str]) = if should_emit_asm {
        (
            &[
                "rustc",
                "--package",
                package,
                "--profile",
                compile_mode,
                "--target",
                &arch_string,
                "-Zbuild-std-features=compiler-builtins-mem",
                "-Zunstable-options",
            ],
            &["--", "--emit", "asm"],
        )
    } else {
        (
            &[
                "build",
                "--package",
                package,
                "--profile",
                compile_mode,
                "--target",
                &arch_string,
                "--artifact-dir",
                "./target/bin",
                "-Zbuild-std-features=compiler-builtins-mem",
                "-Zunstable-options",
            ],
            &[],
        )
    };

    Command::new("cargo")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env("CARGO_TERM_PROGRESS_WHEN", "never")
        .args(pre_build_command)
        .args(feature_flags)
        .args(build_std_options)
        .args(post_build_command)
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
        ArchSelect::I686 | ArchSelect::X64 | ArchSelect::Kernel | ArchSelect::UserSpace => {
            "elf64-x86-64"
        }
    };

    let bin_path = path.with_extension("bin");
    fs::copy(path, &bin_path).context("Failed to duplicate ELF output file")?;

    Command::new("objcopy")
        .args([
            "-I",
            arch,
            "-O",
            "binary",
            bin_path.as_path().to_str().unwrap(),
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
        .truncate(true)
        .write(true)
        .open(&target_location)
        .await?;

    file.write_all(
        br#"bootloader32=/bootloader/stage_32.bin
bootloader64=/bootloader/stage_64.bin
kernel=/kernel.elf
vbe-mode=1280x720
initfs=/initfs
"#,
    )
    .await?;

    Ok(target_location)
}

pub async fn build_initfs_file(initfs_files: &[(PathBuf, PathBuf)]) -> Result<PathBuf> {
    let tar_path = PathBuf::from("./target/bin/initfs");
    let tar_backed = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&tar_path)?;

    let mut ar = tar::Builder::new(tar_backed);

    for (init_elf, to_loc) in initfs_files {
        let mut elf_file = std::fs::OpenOptions::new().read(true).open(init_elf)?;
        ar.append_file(to_loc, &mut elf_file)?;
    }

    ar.finish()?;

    Ok(tar_path)
}

pub async fn file_len_of(file: &Path) -> Result<usize> {
    let file = tokio::fs::OpenOptions::new().read(true).open(file).await?;
    Ok(file.metadata().await?.len() as usize)
}

// FIXME: This 'emit_asm' thing is kinda a hack just to get it working
//        we should change this in the future.
pub async fn build_project(multiboot_mode: bool, emit_asm: Option<String>) -> Result<Artifacts> {
    let (
        stage_bootsector,
        stage_16bit,
        stage_32bit,
        stage_64bit,
        kernel,
        dummy_userspace,
        hello_server,
        boot_cfg,
    ) = tokio::try_join!(
        cargo_helper(
            Some("stage-bootsector"),
            "stage-bootsector",
            ArchSelect::I386,
            None,
            emit_asm.as_ref().is_some_and(|s| s == "stage-bootsector")
        ),
        cargo_helper(
            Some("stage-16bit"),
            "stage-16bit",
            ArchSelect::I386,
            None,
            emit_asm.as_ref().is_some_and(|s| s == "stage-16bit")
        ),
        cargo_helper(
            Some("stage-32bit"),
            "stage-32bit",
            ArchSelect::I686,
            if multiboot_mode {
                Some("multiboot")
            } else {
                None
            },
            emit_asm.as_ref().is_some_and(|s| s == "stage-32bit")
        ),
        cargo_helper(
            Some("stage-64bit"),
            "stage-64bit",
            ArchSelect::X64,
            None,
            emit_asm.as_ref().is_some_and(|s| s == "stage-64bit")
        ),
        cargo_helper(
            Some("kernel"),
            "kernel",
            ArchSelect::Kernel,
            None,
            emit_asm.as_ref().is_some_and(|s| s == "kernel")
        ),
        cargo_helper(
            Some("userspace"),
            "dummy",
            ArchSelect::UserSpace,
            None,
            emit_asm.as_ref().is_some_and(|s| s == "dummy")
        ),
        cargo_helper(
            Some("userspace"),
            "hello-server",
            ArchSelect::UserSpace,
            None,
            emit_asm.as_ref().is_some_and(|s| s == "hello-server")
        ),
        build_bootloader_config(),
    )?;

    let ue_slice = [
        (dummy_userspace, PathBuf::from("./dummy")),
        (hello_server, PathBuf::from("./helloServ")),
        (stage_16bit.clone(), PathBuf::from("./i_should_crash")),
    ];

    let (bootsector, stage_16, stage_32, stage_64, initfs) = tokio::try_join!(
        convert_bin(&stage_bootsector, ArchSelect::I386),
        convert_bin(&stage_16bit, ArchSelect::I386),
        convert_bin(&stage_32bit, ArchSelect::I686),
        convert_bin(&stage_64bit, ArchSelect::X64),
        build_initfs_file(&ue_slice),
    )?;

    let (kernel_len, initfs_len) = tokio::try_join!(file_len_of(&kernel), file_len_of(&initfs))?;

    Ok(Artifacts {
        bootsector,
        stage_16,
        stage_32: if multiboot_mode {
            stage_32bit
        } else {
            stage_32
        },
        stage_64,
        kernel,
        boot_cfg,
        initfs,
        kernel_len,
        initfs_len,
    })
}

pub async fn run_clippy(package: Option<&str>) -> Result<()> {
    let package_args: &[&str] = if let Some(package) = package {
        &["--package", package]
    } else {
        &["--workspace"]
    };

    Command::new("cargo")
        .arg("clippy")
        .args(package_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?
        .success()
        .then_some(())
        .ok_or(Error::msg("Failed to run Cargo"))?;

    Ok(())
}
