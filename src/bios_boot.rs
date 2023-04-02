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

use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::{env, fs};

pub fn build_stage_1() -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let target = format!("{}/target", current_dir.display());
    let cargo = env::var("CARGO").unwrap_or("cargo".into());

    let stage1_path = format!("{}/i386-quantum_loader/release/stage-1", target);

    let cargo_status = Command::new(cargo)
        .current_dir("bootloader/src/bios_boot/stage-1")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("i386-quantum_loader.json")
        .arg(format!("--target-dir={}", target))
        .stdout(std::process::Stdio::piped())
        .status()?;

    if !cargo_status.success() {
        panic!("unable to build bootloader!")
    }

    Command::new("objcopy")
        .arg("-I")
        .arg("elf32-i386")
        .arg("-O")
        .arg("binary")
        .arg(&stage1_path)
        .stdout(std::process::Stdio::piped())
        .status()?;

    Ok(stage1_path)
}

pub fn make_bootloader_dir(path: &String) -> Result<String, Box<dyn std::error::Error>> {
    let bootloader_dir = format!("{}/bootloader_dir", path);

    fs::create_dir(&bootloader_dir)?;

    Ok(bootloader_dir)
}

pub struct BiosBootConfig {
    pub stage2_filepath: String,
    pub kernel_address: String,
    pub kernel_filepath: String,
    pub video_mode_preferred: (usize, usize),
}

impl BiosBootConfig {
    const KERNEL_FILE_LOCATION_KEY: &'static str = "KERNEL_ELF";
    const KERNEL_START_LOCATION_KEY: &'static str = "KERNEL_BEGIN";
    const NEXT_STAGE_LOCATION_KEY: &'static str = "NEXT_STAGE_BIN";
    const VIDEO_MODE_KEY: &'static str = "VIDEO";

    pub fn to_string(&self) -> String {
        format!(
            "{}={}\n{}={}\n{}={}\n{}={}x{}\n",
            Self::KERNEL_START_LOCATION_KEY,
            self.kernel_address,
            Self::KERNEL_FILE_LOCATION_KEY,
            self.kernel_filepath,
            Self::NEXT_STAGE_LOCATION_KEY,
            self.stage2_filepath,
            Self::VIDEO_MODE_KEY,
            self.video_mode_preferred.0,
            self.video_mode_preferred.1
        )
    }
}

pub fn make_config_file(
    path: &String,
    config: BiosBootConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let config_file_path = format!("{}/bootloader.cfg", path);

    let mut bootloader_config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&config_file_path)?;

    bootloader_config_file.write_all(config.to_string().as_bytes())?;

    Ok(config_file_path)
}
