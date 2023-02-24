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
use crate::{debug_print, debug_println};

struct RuntimeInfo {
    success_count: usize,
    failed_count: usize
}

impl RuntimeInfo {
    pub fn new() -> Self {
        Self {
            success_count: 0,
            failed_count: 0,
        }
    }

    pub fn get_run_count(&self) -> usize {
        self.success_count + self.failed_count
    }

    pub fn add_success(&mut self) {
        self.success_count += 1;

        self.run_next_test();
    }

    pub fn has_failed(&self) -> bool {
        self.failed_count > 0
    }

    pub fn add_failed(&mut self) {
        self.failed_count += 1;

        self.run_next_test();
    }

    pub fn get_failed(&self) -> usize {
        self.failed_count
    }

    pub fn run_next_test(&self) {
        if let Some(tests) = unsafe { SYSTEM_TESTS } {
            if self.get_run_count() >= tests.len() {
                return;
            }

            tests[self.get_run_count()].run();
        }
    }
}

static mut CURRENT_RUN : RuntimeInfo = RuntimeInfo { success_count: 0, failed_count: 0};
static mut SYSTEM_TESTS: Option<&[&dyn Testable]> = None;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
    where
        T: Fn(),
{
    fn run(&self) {
        let mut current_run = unsafe { &mut CURRENT_RUN };

        debug_print!("{:#4}: {:120} ", current_run.get_run_count() + 1, core::any::type_name::<T>().blue().bold());
        self();
        debug_println!("{}", "OK".bright_green().bold());

        current_run.add_success();
    }
}
pub fn end_tests() {
    let current_run = unsafe { &CURRENT_RUN };

    if !current_run.has_failed() {
        debug_println!("\n{}\n\n", "All tests passed! Exiting...".bright_green().bold());

        exit_qemu(QemuExitCode::Success);
    } else {
        debug_println!("\n{}/{} {}\n\n",
            current_run.get_failed(),
            current_run.get_run_count(),
            "tests have failed!".red());

        exit_qemu(QemuExitCode::Failed);
    }
}


pub fn test_runner(tests: &'static [&dyn Testable]) {
    debug_println!("Running {} tests...", tests.len());

    unsafe { SYSTEM_TESTS = Some(tests) };
    unsafe { &CURRENT_RUN }.run_next_test();

    end_tests();
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    debug_println!("{}\n", "Failed".bright_red().bold());
    debug_println!("{}\n", info.red());

    unsafe { &mut CURRENT_RUN }.add_failed();

    end_tests();

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}