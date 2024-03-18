use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

#[allow(unused)]
enum ArchSelect {
    I386,
    I686,
    X64,
}

impl Display for ArchSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dir = manifest_dir();
        let current_dir = Path::new(dir.as_str());
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

fn cargo_path() -> String {
    env::var("CARGO").unwrap()
}

fn manifest_dir() -> String {
    env::var("CARGO_MANIFEST_DIR").unwrap()
}

fn compile_mode() -> String {
    env::var("PROFILE").unwrap()
}

fn target_dir() -> String {
    env::var("OUT_DIR").unwrap()
}

fn cargo_helper(profile: Option<&str>, package: &str, arch: ArchSelect) -> String {
    let cargo_bin = cargo_path();
    let compile_mode = compile_mode();
    let compile_mode = profile.unwrap_or(compile_mode.as_str());
    let target_dir = target_dir();

    println!("cargo:rerun-if-changed={}", package);

    Command::new(cargo_bin)
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
            target_dir.as_str(),
        ])
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("Failed to build");

    PathBuf::from(target_dir)
        .join("bin")
        .join(package)
        .to_string_lossy()
        .into()
}

fn convert_bin(path: &str, arch: ArchSelect) -> String {
    let arch = match arch {
        ArchSelect::I386 => "elf32-i386",
        _ => todo!("Add more objcopy arches"),
    };

    let bin_path = format!("{}.bin", path);
    fs::copy(path, bin_path.as_str()).unwrap();

    Command::new("objcopy")
        .args(["-I", arch, "-O", "binary", bin_path.as_str()])
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("Failed to Convert to Binary");

    bin_path
}

fn build_stages() {
    let bootsector = convert_bin(
        &cargo_helper(
            Some("stage-bootsector"),
            "stage-bootsector",
            ArchSelect::I386,
        ),
        ArchSelect::I386,
    );

    let stage_16bit = convert_bin(
        &cargo_helper(Some("stage-16bit"), "stage-16bit", ArchSelect::I386),
        ArchSelect::I386,
    );

    let target_dir = PathBuf::from(target_dir()).join("bin");
    fs::create_dir_all(&target_dir).unwrap();
    fs::copy(bootsector, target_dir.join("stage-bootsector.bin")).unwrap();
    fs::copy(stage_16bit, target_dir.join("stage-16bit.bin")).unwrap();

    println!(
        "cargo:rustc-env=STAGE_BOOTSECTOR_PATH={}",
        target_dir.join("stage-bootsector.bin").to_str().unwrap()
    );
    println!(
        "cargo:rustc-env=STAGE_16BIT_PATH={}",
        target_dir.join("stage-16bit.bin").to_str().unwrap()
    );
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linkerscripts");

    build_stages();
}
