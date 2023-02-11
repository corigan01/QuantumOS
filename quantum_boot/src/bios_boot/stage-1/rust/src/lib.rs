/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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


#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]

use core::arch::asm;
use core::fmt;
use core::panic::PanicInfo;
use crate::bios_ints::{BiosInt, TextModeColor};
use crate::bios_video::BiosVideo;
use crate::console::{GlobalPrint};
use core::fmt::{Debug, Write};

mod bios_ints;
mod cpu_regs;
mod console;
mod bios_video;


#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    unsafe {
        asm!(
            "mov ds, {x}",
            "mov ss, {x}",
            x = in(reg) 0x00
        );
    }

    main();

    loop {}
}



fn main() {
    BiosVideo::print_str("This is a super super duper long test of string or thing idk!");
    Biosvideo::print_str("This is a super test!");

    bios_println!("This is a test!");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
