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

use alloc::{format, string::String};
use lldebug::{LogKind, logln};
use quantum_portal::{
    ConnectHandleError, DebugMsgError, ExitReason, MapMemoryError, MemoryLocation,
    MemoryProtections, QuantumPortal, RecvHandleError, SendHandleError, ServeHandleError,
    WaitSignal, server::QuantumPortalServer,
};

use crate::process::{HandleError, Process, scheduler::Scheduler};

#[unsafe(no_mangle)]
extern "C" fn syscall_handler(
    rdi: u64,
    rsi: u64,
    rdx: u64,
    _rsp: u64,
    r8: u64,
    syscall_number: u64,
) -> u64 {
    unsafe {
        crate::syscall_handler::KernelSyscalls::from_syscall(syscall_number, rdi, rsi, rdx, r8)
    }
}

pub struct KernelSyscalls {}

impl QuantumPortalServer for KernelSyscalls {
    fn verify_user_ptr<T: Sized>(_ptr: *const T) -> bool {
        true
    }
}

impl QuantumPortal for KernelSyscalls {
    fn exit(_exit_reason: ExitReason) -> ! {
        Scheduler::crash_current();
        unreachable!();
    }

    fn map_memory(
        _location: MemoryLocation,
        _protections: MemoryProtections,
        _bytes: usize,
    ) -> Result<*mut u8, MapMemoryError> {
        logln!("map_memory todo");
        Scheduler::crash_current();
        unreachable!();
    }

    fn get_pid() -> usize {
        logln!("get_pid todo");
        Scheduler::crash_current();
        unreachable!();
    }

    fn debug_msg(msg: &str) -> Result<(), DebugMsgError> {
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();
        let process_fmt = format!(
            "{:<24}p{:02x}t{:02x}",
            current_thread.process.name, current_thread.process.id, current_thread.id
        );

        lldebug::priv_print(LogKind::Log, &process_fmt, format_args!("{}", msg));

        Ok(())
    }

    fn yield_me() {
        Scheduler::yield_me();
    }

    fn handle_recv(handle: u64, buf: &mut [u8]) -> Result<usize, RecvHandleError> {
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();
        current_thread
            .process
            .handle_rx(handle, buf)
            .map_err(|err| match err {
                HandleError::HandleDoesntExist(_) => RecvHandleError::InvalidHandle,
                HandleError::InvalidSocketKind | HandleError::HostDisconnect => {
                    RecvHandleError::RecvFailed
                }
                HandleError::WouldBlock => RecvHandleError::WouldBlock,
            })
    }

    fn handle_send(handle: u64, buf: &[u8]) -> Result<usize, SendHandleError> {
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();
        current_thread
            .process
            .handle_tx(handle, buf)
            .map_err(|err| match err {
                HandleError::HandleDoesntExist(_) => SendHandleError::InvalidHandle,
                HandleError::InvalidSocketKind | HandleError::HostDisconnect => {
                    SendHandleError::SendFailed
                }
                HandleError::WouldBlock => SendHandleError::WouldBlock,
            })
    }

    fn handle_serve(endpoint: &str) -> Result<u64, ServeHandleError> {
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();
        Process::new_connection_handle(current_thread.process.clone(), String::from(endpoint))
            .ok_or(ServeHandleError::AlreadyBound)
    }

    fn handle_connect(endpoint: &str) -> Result<u64, ConnectHandleError> {
        let s = Scheduler::get();
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();

        // Get the handle owner
        let Some((owner, owner_id)) = s.serve_sockets.lock().get(endpoint).cloned() else {
            return Err(ConnectHandleError::EndpointDoesNotExist);
        };
        let Some(owner) = owner.upgrade() else {
            s.serve_sockets.lock().remove(endpoint);
            return Err(ConnectHandleError::EndpointDoesNotExist);
        };

        let (_, client_handle_id) =
            Process::new_handle_pair(owner, owner_id, current_thread.process.clone());
        Ok(client_handle_id)
    }

    fn handle_disconnect(handle: u64) {
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();
        Process::disconnect_handle(current_thread.process.clone(), handle);
    }

    fn signal_wait() -> WaitSignal {
        let current_thread = Scheduler::get().current_thread().upgrade().unwrap();
        current_thread.process.next_signal()
    }
}
