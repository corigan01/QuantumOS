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
    ExitReason, RecvHandleError, dbugln, exit, handle_connect, handle_disconnect, handle_recv,
    handle_send, yield_me,
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dbugln!("{}", info);
    loop {}
}

#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
extern "C" fn _start() {
    dbugln!("Starting Dummy!");

    let handle = loop {
        match handle_connect("hello-server") {
            Ok(okay) => break okay,
            Err(e) => {
                dbugln!("Error connecting to `hello-server` {e:?}...");
                yield_me();
            }
        }
    };

    handle_send(handle, &[0xde, 0xad, 0xbe, 0xef]).unwrap();
    let mut data_buf = [0; 32];
    loop {
        match handle_recv(handle, &mut data_buf) {
            Ok(b) => {
                dbugln!("Got {b} bytes! {data_buf:04x?}");
            }
            Err(RecvHandleError::WouldBlock) => (),
            Err(err) => {
                dbugln!("Error {err:?}!");
                break;
            }
        }
    }
    handle_disconnect(handle);
    exit(ExitReason::Success);
}
