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

use crate::artifacts::get_program_path;
use crate::CompileOptions;
use anyhow::{anyhow, Context};
use std::process::Command;

pub fn spawn_qemu(disk_target_path: &String, options: &CompileOptions) -> anyhow::Result<i32> {
    let qemu_exe = get_program_path("qemu-system-x86_64")?;

    let mut user_extra_args = vec![];

    if options.kvm {
        user_extra_args.push("-enable-kvm");
    }

    if options.options.get_run_options().unwrap().headless {
        user_extra_args.push("-nographic");
        user_extra_args.push("-serial");
        user_extra_args.push("mon:stdio");
    } else {
        user_extra_args.push("-serial");
        user_extra_args.push("stdio");
        user_extra_args.push("-display");
        user_extra_args.push("gtk");
    }

    let qemu = Command::new(qemu_exe)
        .args(user_extra_args)
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
        .arg("-d")
        .arg("int")
        .arg("--no-reboot")
        .arg("-m")
        .arg("256M")
        .arg("-k")
        .arg("en-us")
        .arg("-nic")
        .arg("none")
        .arg("-drive")
        .arg(format!("format=raw,file={}", disk_target_path))
        .stdout(std::process::Stdio::inherit())
        .status()
        .context(anyhow!("Could not start qemu-system-x86_64!"))?;

    Ok(qemu.code().ok_or(anyhow!("Could not get qemu exit code"))?)
}
