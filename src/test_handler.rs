/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

Quantum OS Lib file, documentation coming soon!

*/

use core::panic::PanicInfo;
use owo_colors::OwoColorize;
use crate::qemu::{exit_qemu, QemuExitCode};
use crate::{debug_print, debug_println, serial_print, serial_println};
use spin::Mutex;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};
use lazy_static::lazy_static;

struct RuntimeInfo {
    run_count: usize
}

impl RuntimeInfo {
    pub fn new() -> Self {
        Self {
            run_count: 0
        }
    }

    pub fn get_run_count(&self) -> usize {
        self.run_count
    }

    pub fn add_run(&mut self) {
        self.run_count += 1
    }
}


lazy_static! {
    static ref CURRENT_RUN : Mutex<RuntimeInfo> = {
        Mutex::new(RuntimeInfo::new())
    };
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
    where
        T: Fn(),
{
    fn run(&self) {
        let mut current_run = CURRENT_RUN.lock();

        debug_print!("{:#4}: {:120} ", current_run.get_run_count(), core::any::type_name::<T>().blue().bold());
        self();
        debug_println!("{}", "OK".bright_green().bold());

        current_run.add_run();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    debug_println!("Running {} tests...", tests.len());

    for test in tests {
        test.run();
    }

    debug_println!("\n{}", "All tests passed! Exiting...".bright_green().bold());

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    debug_println!("{}\n", "Failed".bright_red().bold());
    debug_println!("{}", info.red());
    debug_println!("\n\n-------------------------------");


    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}