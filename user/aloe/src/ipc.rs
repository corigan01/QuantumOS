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

extern crate alloc;

use portal::ipc::{IpcError, IpcResult};
use vera_portal::{
    ConnectHandleError, HandleUpdateKind, RecvHandleError, SendHandleError, ServeHandleError,
    WaitSignal,
    sys_client::{close, connect, recv, send, serve, yield_now},
};

pub struct QuantumGlue(u64);

impl QuantumGlue {
    pub const fn new(handle: u64) -> Self {
        Self(handle)
    }

    /// A blocking connect to the service
    pub fn connect_to(service: &str) -> IpcResult<Self> {
        let handle = loop {
            match connect(service) {
                Ok(okay) => {
                    break okay;
                }
                Err(ConnectHandleError::EndpointDoesNotExist) => {
                    yield_now();
                }
            }
        };

        Ok(Self::new(handle))
    }
}

impl portal::ipc::IpcGlue for QuantumGlue {
    fn disconnect(&mut self) {
        close(self.0);
    }
}

impl portal::ipc::Sender for QuantumGlue {
    fn send(&mut self, bytes: &[u8]) -> IpcResult<()> {
        send(self.0, bytes).map_err(|send_err| match send_err {
            SendHandleError::InvalidHandle | SendHandleError::SendFailed => IpcError::GlueError,
            SendHandleError::WouldBlock => IpcError::NotReady,
        })?;

        Ok(())
    }
}

impl portal::ipc::Receiver for QuantumGlue {
    fn recv(&mut self, bytes: &mut [u8]) -> IpcResult<usize> {
        recv(self.0, bytes).map_err(|recv_err| match recv_err {
            RecvHandleError::InvalidHandle | RecvHandleError::RecvFailed => IpcError::GlueError,
            RecvHandleError::WouldBlock => IpcError::NotReady,
        })
    }
}

/// Server host component
pub struct QuantumHost<T> {
    server_handle: u64,
    mapping: alloc::collections::BTreeMap<u64, T>,
}

impl<T> QuantumHost<T> {
    pub fn host_on(service_name: &str) -> IpcResult<Self> {
        match serve(service_name) {
            Ok(server_handle) => Ok(Self {
                server_handle,
                mapping: alloc::collections::BTreeMap::new(),
            }),
            Err(ServeHandleError::AlreadyBound) => Err(IpcError::AlreadyUsed),
        }
    }

    pub fn service_signal<N, R, W, D>(
        &mut self,
        signal: WaitSignal,
        new_fn: N,
        read_fn: R,
        write_fn: W,
        disconnect_fn: D,
    ) -> IpcResult<()>
    where
        N: FnOnce(u64) -> IpcResult<T>,
        R: FnOnce(&mut T) -> IpcResult<()>,
        W: FnOnce(&mut T) -> IpcResult<()>,
        D: FnOnce(T) -> IpcResult<()>,
    {
        match signal {
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::NewConnection { new_handle },
            } if handle == self.server_handle => {
                self.mapping.insert(new_handle, new_fn(new_handle)?);
                Ok(())
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::Disconnected,
            } => {
                if let Some(prev) = self.mapping.remove(&handle) {
                    disconnect_fn(prev)?;
                }

                Ok(())
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::ReadReady,
            } => {
                if let Some(mapping_inner) = self.mapping.get_mut(&handle) {
                    read_fn(mapping_inner)?;
                }

                Ok(())
            }
            WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::WriteReady,
            } => {
                if let Some(mapping_inner) = self.mapping.get_mut(&handle) {
                    write_fn(mapping_inner)?;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
