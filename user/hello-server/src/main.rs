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

use alloc::collections::btree_map::BTreeMap;
use hello_portal::HelloPortalServer;
use libq::{
    HandleUpdateKind, RecvHandleError, SendHandleError, WaitSignal, close, dbugln, recv, send,
    serve, signal_wait, tiny_std,
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
        send(self.0, bytes).map_err(|send_err| match send_err {
            SendHandleError::InvalidHandle => panic!("Invalid handle!"),
            SendHandleError::SendFailed => IpcError::GlueError,
            SendHandleError::WouldBlock => IpcError::NotReady,
        })?;

        Ok(())
    }
}

impl portal::ipc::Receiver for QuantumGlue {
    fn recv(&mut self, bytes: &mut [u8]) -> IpcResult<usize> {
        recv(self.0, bytes).map_err(|recv_err| match recv_err {
            RecvHandleError::InvalidHandle => panic!("Invalid handle!"),
            RecvHandleError::RecvFailed => IpcError::GlueError,
            RecvHandleError::WouldBlock => IpcError::NotReady,
        })
    }
}

fn main() {
    dbugln!("Hello Server!");
    let server_handle = serve("hello-server").unwrap();
    let mut handle_servers = BTreeMap::new();

    loop {
        let signal = signal_wait();
        // dbugln!("Wakeup with signal ({signal:?})");

        match signal {
            WaitSignal::HandleUpdate {
                kind: HandleUpdateKind::NewConnection { new_handle },
                handle,
            } if handle == server_handle => {
                handle_servers.insert(new_handle, HelloPortalServer::new(QuantumGlue(new_handle)));
                dbugln!("New connection - handle={new_handle} ");
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::ReadReady,
            } => {
                if let Some(serv) = handle_servers.get_mut(&handle) {
                    dbugln!("Incoming - handle={handle}");
                    match serv.incoming() {
                        Ok(hello_portal::HelloPortalClientRequest::PingHelloServer { sender }) => {
                            dbugln!("HELLO!");
                            sender.respond_with(true).unwrap();
                        }
                        Err(err) => dbugln!("IpcError = {:#?}", err),
                        _ => todo!(),
                    }
                }
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::WriteReady,
            } => {
                // let data_buf = [0xde, 0xea, 0xbe, 0xef];
                // let valid_bytes = send(handle, &data_buf).unwrap();

                // dbugln!(
                //     "Sent {valid_bytes} from {handle}: {:02x?}",
                //     &data_buf[..valid_bytes]
                // );
            }
            WaitSignal::TerminationRequest => {
                dbugln!("Quitting...");
                return;
            }
            _ => (),
        }
    }
}
