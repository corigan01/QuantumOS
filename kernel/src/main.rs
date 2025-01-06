/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2025 Gavin Kellam

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

#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

mod panic;

use bootloader::KernelBootHeader;
use lldebug::{debug_ready, logln, make_debug};
use serial::{Serial, baud::SerialBaud};
use util::bytes::HumanBytes;

make_debug! {
    "Serial": Option<Serial> = Serial::probe_first(SerialBaud::Baud115200);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
extern "C" fn _start(kbh: u64) -> ! {
    main(unsafe { &*(kbh as *const KernelBootHeader) });
    panic!("Main should not return");
}

#[debug_ready]
fn main(kbh: &KernelBootHeader) {
    logln!("Welcome to the Quantum Kernel!");
    logln!(
        "Free Memory : {}",
        HumanBytes::from(kbh.phys_mem_map.bytes_of(mem::phys::PhysMemoryKind::Free))
    );
}
