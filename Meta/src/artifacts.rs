/*
  ____                 __
 / __ \__ _____ ____  / /___ ____ _
/ /_/ / // / _ `/ _ \/ __/ // /  ' \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use std::{env, fs};
use std::fs::remove_dir_all;
use anyhow::{anyhow, Context};
use std::path::Path;
use std::process::Command;
use crate::{CompileOptions, RunCommands};
use owo_colors::OwoColorize;

pub fn ensure_artifact_dir_exists(path: &Path) -> anyhow::Result<()> {
    if path.is_file() {
        fs::remove_file(path)?;
    }

    if path.exists() {
        return Ok(());
    }

    fs::create_dir(path)?;

    Ok(())
}

/// # Get Program Path
/// Finds the true path of any program in the $PATH
pub fn get_program_path(program_name: &str) -> anyhow::Result<String> {
    env::var("PATH")
        .map_err(|_| anyhow!("Could not get PATH"))?
        .split(":")
        .find_map(|path| {
            let path = Path::new(path);
            path.read_dir().ok()?.find_map(|entry| {
                let entry = entry.ok()?;

                if entry.file_name() == program_name {
                    Some(String::from(entry.path().to_string_lossy()))
                } else {
                    None
                }
            })
        })
        .ok_or(anyhow!(
            "Could not find '{program_name}' in PATH!"
        ))
        .with_context(|| {
        anyhow!("{}The program '{}' is required to build QuantumOS! Refer to your system package manager for instructions on how to install '{}'.", "".bold(), program_name.bold(), program_name.bold())
    })
}

/// # Get Cargo Path
/// Finds the true cargo path from the $PATH env variable. A simple 'whereis' in rust.
pub fn get_cargo_path() -> anyhow::Result<String> {
    get_program_path("cargo")
}


/// # Does Directory Contain File
/// Checks if a dir has a 'filename'-file child.
pub fn does_directory_contain_file(dir_path: &str, filename: &str) -> anyhow::Result<bool> {
    let dir = Path::new(dir_path);
    Ok(dir.read_dir()?
        .find(|file| {
            if let Ok(file) = file {
                file.file_name() == filename
            } else { false }
        })
        .ok_or(anyhow!("Could not find file in path"))?
        .map(|_| true)?)
}

/// # Get Project Root
/// Gets the root of the project directory. Grabs the absolute path of 'QuantumOS'.
pub fn get_project_root() -> anyhow::Result<String> {
    let current_dir = env::current_dir()
        .map_err(|_| anyhow!("Could not find current directory"))?;

    let attempted_root =
        current_dir
            .to_string_lossy()
            .split("Meta")
            .next()
            .map(|str| String::from(str))
            .ok_or(anyhow!("Could not determine path of project root"))?;

    if !does_directory_contain_file(attempted_root.as_str(), "Meta")? ||
        !does_directory_contain_file(attempted_root.as_str(), "kernel")? {
        return Err(anyhow!("Attempted project root does not contain './Meta/' or './Kernel/', which should not be possible"))
    }

    Ok(String::from(attempted_root))
}

/// # Get Target Directory
/// Gets the absolute path of the target directory.
pub fn get_target_directory() -> anyhow::Result<String> {
    Ok(format!("{}target", get_project_root()?))
}

pub fn build_kernel(options: &CompileOptions) -> anyhow::Result<String> {
    let cargo_path = get_cargo_path()?;
    let project_root = get_project_root()?;

    let kernel_root = format!("{}/kernel", project_root);
    let target_root = get_target_directory()?;
    let target_kernel_root = format!("{}/kernel", target_root);

    let release_mode = if options.debug_compile { "dev" } else { "release" };
    let build_or_test = if matches!(options.options, RunCommands::Test(_)) { "test" } else { "build" };

    let kernel_build = Command::new(&cargo_path)
        .current_dir(kernel_root)
        .arg(build_or_test)
        .arg("--profile")
        .arg(release_mode)
        .arg("--target")
        .arg("x86_64-quantum_os.json")
        .arg(format!("--target-dir={}", target_kernel_root))
        .stdout(std::process::Stdio::piped())
        .status()?;

    if !kernel_build.success() {
        return Err(anyhow!("Could not compile QuantumOS"));
    }

    let exe_path = format!("{}/x86_64-quantum_os/{}/quantum_os",
                           target_kernel_root,
                           if options.debug_compile { "debug" } else { "release" }
    );

    Ok(exe_path)
}

pub fn build_bios_stage_1() -> anyhow::Result<String> {
    let cargo_dir = get_cargo_path()?;
    let target_root = get_target_directory()?;
    let project_root = get_project_root()?;

    let stage_1_root = format!("{}/bootloader/src/bios_boot/stage-1", project_root);
    let stage_1_target_root = format!("{}/stage1", target_root);

    // This stage *MUST* be in release mode to fit in 64k
    let stage_1_build = Command::new(&cargo_dir)
        .current_dir(stage_1_root)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("i386-quantum_loader.json")
        .arg(format!("--target-dir={}", stage_1_target_root))
        .stdout(std::process::Stdio::piped())
        .status()?;

    if !stage_1_build.success() {
        return Err(anyhow!("Could not compile Stage1"));
    }

    let stage1_path = format!("{}/i386-quantum_loader/release/stage-1", stage_1_target_root);

    Command::new("objcopy")
        .arg("-I")
        .arg("elf32-i386")
        .arg("-O")
        .arg("binary")
        .arg(&stage1_path)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()?;

    Ok(stage1_path)
}

pub fn build_bios_stage_2(options: &CompileOptions) -> anyhow::Result<String> {
    let cargo_dir = get_cargo_path()?;
    let target_root = get_target_directory()?;
    let project_root = get_project_root()?;

    let stage_2_root = format!("{}/bootloader/src/bios_boot/stage-2", project_root);
    let stage_2_target_root = format!("{}/stage2", target_root);

    // FIXME: We should try to get stage2 working in debug mode
    const CAN_STAGE2_WORK_IN_DEBUG: bool = false;

    let release_mode = if options.debug_compile && CAN_STAGE2_WORK_IN_DEBUG { "dev" } else { "release" };

    let stage_2_build = Command::new(&cargo_dir)
        .current_dir(stage_2_root)
        .arg("build")
        .arg("--profile")
        .arg(release_mode)
        .arg("--target")
        .arg("i686-quantum_loader.json")
        .arg(format!("--target-dir={}", stage_2_target_root))
        .stdout(std::process::Stdio::piped())
        .status()?;

    if !stage_2_build.success() {
        return Err(anyhow!("Could not compile Stage2"));
    }

    let exe_path = format!("{}/i686-quantum_loader/{}/stage-2",
                           stage_2_target_root,
                           if options.debug_compile && CAN_STAGE2_WORK_IN_DEBUG { "debug" } else { "release" }
    );

    assert!(Path::new(&exe_path).exists(), "Stage2 compiled, but does not exist at path {}", exe_path);

    Command::new("objcopy")
        .arg("-I")
        .arg("elf64-x86-64")
        .arg("--binary-architecture=i386:x86-64")
        .arg("-O")
        .arg("binary")
        .arg(&exe_path)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()?;

    Ok(exe_path)
}

pub fn build_bios_stage_3(options: &CompileOptions) -> anyhow::Result<String> {
    let cargo_dir = get_cargo_path()?;
    let target_root = get_target_directory()?;
    let project_root = get_project_root()?;

    let stage_3_root = format!("{}/bootloader/src/bios_boot/stage-3", project_root);
    let stage_3_target_root = format!("{}/stage3", target_root);

    let release_mode = if options.debug_compile { "dev" } else { "release" };

    let stage_3_build = Command::new(&cargo_dir)
        .current_dir(stage_3_root)
        .arg("build")
        .arg("--profile")
        .arg(release_mode)
        .arg("--target")
        .arg("x86_64-quantum_loader.json")
        .arg(format!("--target-dir={}", stage_3_target_root))
        .stdout(std::process::Stdio::piped())
        .status()?;

    if !stage_3_build.success() {
        return Err(anyhow!("Could not compile Stage3"));
    }

    let exe_path = format!("{}/x86_64-quantum_loader/{}/stage-3",
                           stage_3_target_root,
                           if options.debug_compile { "debug" } else { "release" }
    );

    Command::new("objcopy")
        .arg("-I")
        .arg("elf64-x86-64")
        .arg("--binary-architecture=i386:x86-64")
        .arg("-O")
        .arg("binary")
        .arg(&exe_path)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()?;

    assert!(Path::new(&exe_path).exists(), "Stage3 compiled, but does not exist at path {}", exe_path);

    Ok(exe_path)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StageID {
    Stage1,
    Stage2,
    Stage3
}

impl StageID {
    pub fn to_stage_id_string(self) -> String {
        match self {
            StageID::Stage1 => { String::from("stage-1") }
            StageID::Stage2 => { String::from("stage-2") }
            StageID::Stage3 => { String::from("stage-3") }
        }
    }
}

pub fn build_bios_bootloader_items(options: &CompileOptions) -> anyhow::Result<Vec<(StageID, String)>> {
    let mut items: Vec<(StageID, String)> = Vec::new();

    items.push((StageID::Stage1, build_bios_stage_1()?));
    items.push((StageID::Stage2, build_bios_stage_2(options)?));
    items.push((StageID::Stage3, build_bios_stage_3(options)?));

    Ok(items)
}

pub fn remove_target_root() -> anyhow::Result<()> {
    Ok(remove_dir_all(get_target_directory()?)?)
}

#[cfg(test)]
mod test {
    use crate::artifacts::{get_cargo_path, get_project_root};

    #[test]
    fn does_find_cargo_path() {
        let cargo_path = get_cargo_path();
        assert!(cargo_path.is_ok(), "{:?}", cargo_path);
    }

    #[test]
    fn test_find_project_root() {
        let project_root = get_project_root();
        assert!(project_root.is_ok(), "{:?}", project_root);

    }
}