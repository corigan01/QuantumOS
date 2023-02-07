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


const VGA_ADDRESS: *mut u8 = unsafe {0xb8000 as *mut u8};

/*
// define our structure
typedef struct __attribute__ ((packed)) {
    unsigned short di, si, bp, sp, bx, dx, cx, ax;
    unsigned short gs, fs, es, ds, eflags;
} regs16_t;

// tell compiler our int32 function is external
extern void int32(unsigned char intnum, regs16_t *regs);
 */

#[repr(C, packed)]
pub struct Regs16 {
    di: u16,
    si: u16,
    bp: u16,
    sp: u16,
    bx: u16,
    dx: u16,
    cx: u16,
    ax: u16,

    gs: u16,
    fs: u16,
    es: u16,
    ds: u16,
    eflags: u16
}

impl Regs16 {
    pub fn new() -> Self {
        Regs16 {
            di: 0,
            si: 0,
            bp: 0,
            sp: 0,
            bx: 0,
            dx: 0,
            cx: 0,
            ax: 0,
            gs: 0,
            fs: 0,
            es: 0,
            ds: 0,
            eflags: 0,
        }
    }
}

extern "C" { pub fn int32(int: u8, reg: u32); }

fn test(byte: char) {
    unsafe { *VGA_ADDRESS = byte as u8; };
}



/*

    regs.ax = 0x0013;
    int32(0x10, &regs);

    // full screen with blue color (1)
    memset((char *)0xA0000, 1, (320*200));
 */

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    for i in 0_u32..0x8000 {
        let ptr = i as *mut u8;

        unsafe { *ptr = 0x00; }
    }



    let mut regs = Regs16::new();
    //regs.ax = 0x0013;

    test('a');
    unsafe {
        int32(0x10, &regs as *const _ as u32);
    }
    test('b');

    /*for i in 0xA0000_u32..(0xA0000 + (320*200)) {
        let ptr = i as *mut u8;

        unsafe { *ptr = 0x01; }
    }*/


    loop {}
}


use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}