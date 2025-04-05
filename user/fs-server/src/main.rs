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
tiny_std!();

use fs_portal::FsPortalServer;
use aloe::{
    dbugln,
    ipc::{QuantumGlue, QuantumHost},
    signal_wait, tiny_std,
};

mod ata;

fn main() {
    dbugln!("Starting Filesystem server!");

    let mut server = QuantumHost::<FsPortalServer<QuantumGlue>>::host_on("fs").unwrap();
    loop {
        let signal = signal_wait();

        server
            .service_signal(
                signal,
                |handle| Ok(FsPortalServer::new(QuantumGlue::new(handle))),
                |read_cs| match read_cs.incoming()? {
                    fs_portal::FsPortalClientRequest::Ping { sender } => {
                        dbugln!("Got Ping, responding with Pong!");
                        sender.respond_with(())
                    }
                    _ => Ok(()),
                },
                |_| Ok(()),
                |_| {
                    dbugln!("Disconnecting Client");
                    Ok(())
                },
            )
            .unwrap();
    }
}
