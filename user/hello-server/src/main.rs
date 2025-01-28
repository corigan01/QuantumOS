/*
  ____                 __               __  __
 / __ \__ _____ ____  / /___ ____ _    / / / /__ ___ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /_/ (_-</ -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/  \____/___/\__/_/
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

use core::panic::PanicInfo;
use libq::{
    ExitCode, dbugln, exit,
    syscall::{MemoryLocation, MemoryProtections, map_memory},
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dbugln!("{}", info);
    loop {}
}

#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
extern "C" fn _start() {
    dbugln!("Hello Server!");

    let example_memory =
        unsafe { map_memory(MemoryLocation::Anywhere, MemoryProtections::ReadWrite, 10) }.unwrap();

    let slice = unsafe { core::slice::from_raw_parts_mut(example_memory, 10) };
    slice
        .iter_mut()
        .enumerate()
        .for_each(|(i, el)| *el = i as u8);

    dbugln!("{:?}", slice);

    exit(ExitCode::Success);
}
