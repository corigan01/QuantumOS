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

use std::process::exit;
use clap::{Args, Parser, Subcommand, ValueEnum};
use indicatif::{MultiProgress, ProgressStyle};
use owo_colors::OwoColorize;
use crate::artifacts::{build_bios_bootloader_items, build_kernel, get_target_directory, remove_target_root};
use crate::emulator_spawner::spawn_qemu;
use crate::filesystem_constructor::make_and_construct_bios_image;

mod artifacts;
mod filesystem_constructor;
mod config_generator;
mod emulator_spawner;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum BootloaderOption {
    /// Use bios booting
    Bios,
    /// use uefi booting
    Uefi
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand, Debug)]
pub enum RunCommands {
    /// Build QuantumOS and all of its dependencies
    Build,
    /// Run QuantumOS
    Run(RunOptions),
    /// Test QuantumOS
    Test(TestOptions),
    /// Delete Build artifacts
    Clean
}

impl RunCommands {
    pub fn get_run_options(&self) -> Option<RunOptions> {
        match self {
            Self::Run(options) => { Some(*options) },
            _ => { None }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Args, Debug)]
pub struct RunOptions {
    /// Choice to run QuantumOS without a visible qemu window
    #[arg(long)]
    pub headless: bool,

    /// Should Skip building
    #[arg(short = 's', long)]
    pub skip_build: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Args, Debug)]
pub struct TestOptions {
    /// Test the libraries
    #[arg(short = 'l', long, default_value_t = true)]
    pub test_libs: bool,

    /// Test the kernel
    #[arg(short = 'k', long)]
    pub test_kernel: bool,
}

/// Meta QuantumOS Compile Script
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CompileOptions {
    /// Chose how to use QuantumOS
    #[command(subcommand)]
    pub options: RunCommands,

    /// Which bootloader to use (bios / uefi)
    #[arg(short, long, value_enum, default_value_t = BootloaderOption::Bios)]
    pub bootloader: BootloaderOption,

    /// Enable KVM
    #[arg(long)]
    pub kvm: bool,

    /// Debug Compile Mode
    #[arg(short, long)]
    pub debug_compile: bool
}

#[macro_export]
macro_rules! status_print {
    ($($arg:tt)*) => {
        print!("    {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! status_println {
    () => ($crate::status_print!("\n"));
    ($($arg:tt)*) => {
        $crate::status_print!("{}", format_args!($($arg)*));
        $crate::status_print!("\n");
    }
}

const BUILD_ARTIFACTS_DIR: &str = "target";

pub fn build(options: &CompileOptions) {
    status_println!("Building QuantumOS");



    let kern = build_kernel(options).unwrap();

    if options.bootloader == BootloaderOption::Bios {
        let bios = build_bios_bootloader_items(options).unwrap();


        make_and_construct_bios_image(&kern, &bios).unwrap();
    } else {
        todo!("Make UEFI bootloader!");
    }
}

pub fn run(options: &CompileOptions) {
    status_println!("Running QuantumOS");

    let disk_path = format!("{}/disk.img", get_target_directory().unwrap());
    let qemu_status = spawn_qemu(&disk_path, options).unwrap();

    if qemu_status != 33 {
        exit(qemu_status);
    }
}

pub fn test(options: &CompileOptions) {
    match options.options {
        RunCommands::Test(test_options) => {
            if test_options.test_kernel {
                test_kernel();
            }
            if test_options.test_libs {
                test_libs();
            }

            if !test_options.test_libs && !test_options.test_kernel {
                status_println!("Nothing to do!");
            }
        }
        _ => { unreachable!("Should not be possible to reach"); }
    }
}

pub fn test_kernel() {
    status_println!("Testing Kernel");
}

pub fn test_libs() {
    status_println!("Testing Libs");
}

pub fn clean() {
    status_println!("Cleaning Artifacts");
    remove_target_root().unwrap();
}