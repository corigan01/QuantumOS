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

use hello_portal::HelloPortalClient;
use libq::{
    RecvHandleError, SendHandleError, close, connect, dbugln, recv, send, tiny_std, yield_now,
};
use portal::ipc::{IpcError, IpcResult};

pub struct QuantumGlue(u64);

impl portal::ipc::IpcGlue for QuantumGlue {
    fn disconnect(&mut self) {
        close(self.0);
    }
}

impl portal::ipc::Sender for QuantumGlue {
    fn send(&mut self, bytes: &[u8]) -> IpcResult<()> {
        dbugln!("Sending {:?}", bytes);
        send(self.0, bytes).map_err(|send_err| match send_err {
            SendHandleError::InvalidHandle | SendHandleError::SendFailed => IpcError::GlueError,
            SendHandleError::WouldBlock => IpcError::NotReady,
        })?;

        Ok(())
    }
}

impl portal::ipc::Receiver for QuantumGlue {
    fn recv(&mut self, bytes: &mut [u8]) -> IpcResult<usize> {
        recv(self.0, bytes)
            .map_err(|recv_err| match recv_err {
                RecvHandleError::InvalidHandle | RecvHandleError::RecvFailed => IpcError::GlueError,
                RecvHandleError::WouldBlock => IpcError::NotReady,
            })
            .inspect(|&amount| dbugln!("Recv {:?}", &bytes[..amount]))
    }
}

fn main() {
    dbugln!("Starting Dummy");

    let handle = loop {
        match connect("hello-server") {
            Ok(okay) => {
                dbugln!("Connected to `hello-server` at handle {okay}!");
                break okay;
            }
            Err(e) => {
                dbugln!("Error connecting to `hello-server` {e:?}...");
                yield_now();
            }
        }
    };

    let quantum_handle = QuantumGlue(handle);
    let mut portal = HelloPortalClient::new(quantum_handle);

    loop {
        dbugln!("{:?}", portal.ping_hello_server_blocking());
        yield_now();
    }
}
