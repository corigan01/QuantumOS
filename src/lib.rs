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

use std::env;
use std::process::Command;

pub mod bios_boot;
pub mod bios_disk;

pub fn get_build_directory() -> Result<String, Box<dyn std::error::Error>> {
    let current_directory = env::current_dir()?;
    Ok(format!("{}/target", current_directory.display()))
}

pub fn build_kernel() -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let target = format!("{}/target/kernel", current_dir.display());
    let cargo = env::var("CARGO").unwrap_or("cargo".into());

    let kernel_path = format!("{}/x86_64-quantum_os/release/quantum_os", target);

    let cargo_status = Command::new(cargo)
        .current_dir("kernel/")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-quantum_os.json")
        .arg(format!("--target-dir={}", target))
        .stdout(std::process::Stdio::piped())
        .status()?;

    if !cargo_status.success() {
        panic!("unable to build bootloader!")
    }

    Ok(kernel_path)
}
