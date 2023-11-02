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

use meta::{BootloaderOption, build, clean, CompileOptions, run, RunCommands, status_println, test};
use clap::Parser;

/// # QuantumOS Meta
/// This is the compile script for quantum os. You can think of this project like a custom
/// version of 'make' or 'cmake' as it serves the same purpose. This takes all of the
/// components of quantum and combines them together into something that runs. This script
/// is usually going to be your portal into interacting with Quantum as it will handle all
/// of the complicated dependencies and filesystem imaging that goes with building an OS.
fn main() {
    let args = CompileOptions::parse();

    assert_ne!(args.bootloader, BootloaderOption::Uefi,
        "Booting in uefi is not currently supported!"
    );

    status_println!("Quantum Builder");

    // # Options
    // These are the options 'Meta' supports, so we switch them here to call the actual
    // function that handles that operation
    match args.options {
        RunCommands::Build => {
            build(&args);
        }
        RunCommands::Run(run_options) => {
            if !run_options.skip_build {
                build(&args);
            }
            run(&args);
        }
        RunCommands::Test(_) => {
            test(&args);
        }
        RunCommands::Clean => {
            clean();
        }
    }
}
