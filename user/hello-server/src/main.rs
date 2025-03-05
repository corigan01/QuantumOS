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
    HandleUpdateKind, WaitSignal, dbugln, exit, handle_recv, handle_send, handle_serve, signal_wait,
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dbugln!("{}", info);
    exit(libq::ExitReason::Failure);
}

#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
extern "C" fn _start() {
    dbugln!("Hello Server!");
    let server_handle = handle_serve("hello-server").unwrap();

    loop {
        let signal = signal_wait();
        dbugln!("Wakeup with signal ({signal:?})");

        match signal {
            WaitSignal::HandleUpdate {
                kind: HandleUpdateKind::NewConnection { new_handle },
                handle,
            } if handle == server_handle => {
                dbugln!("New connection : {new_handle} ");
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::ReadReady,
            } => {
                let mut data_buf = [0; 16];
                let valid_bytes = handle_recv(handle, &mut data_buf).unwrap();

                dbugln!(
                    "Got {valid_bytes} from {handle}: {:02x?}",
                    &data_buf[..valid_bytes]
                );
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::WriteReady,
            } => {
                let data_buf = [0xde, 0xea, 0xbe, 0xef];
                let valid_bytes = handle_send(handle, &data_buf).unwrap();

                dbugln!(
                    "Sent {valid_bytes} from {handle}: {:02x?}",
                    &data_buf[..valid_bytes]
                );
            }
            WaitSignal::TerminationRequest => {
                dbugln!("Quitting...");
                exit(libq::ExitReason::Success);
            }
            _ => (),
        }
    }
}
