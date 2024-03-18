use std::env;
use std::fmt::Display;
use std::path::Path;
use std::process::Command;

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

fn cargo_helper(profile: Option<&str>, package: &str, arch: ArchSelect) -> String {
    let cargo_bin = cargo_path();
    let compile_mode = compile_mode();
    let compile_mode = profile.unwrap_or(compile_mode.as_str());

    Command::new(cargo_bin)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .args([
            "build",
            "--package",
            package,
            "--profile",
            compile_mode,
            "--target",
            arch.to_string().as_str(),
        ])
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("Failed to build");

    "Test".into()
}

fn main() {
    let root_crate_test = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if root_crate_test == "none" {
        return;
    }

    let _build_bootsector = cargo_helper(
        Some("stage-bootsector"),
        "stage-bootsector",
        ArchSelect::I386,
    );
}
