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

use quantum_portal::{
    DebugMsgError, ExitReason, MapMemoryError, MemoryLocation, MemoryProtections,
};

use crate::process::send_scheduler_event;

pub struct KernelSyscalls {}

impl quantum_portal::server::QuantumPortalServer for KernelSyscalls {
    fn verify_user_ptr<T: Sized>(ptr: *const T) -> bool {
        // TODO: Add ptr verify
        true
    }
}

impl quantum_portal::QuantumPortal for KernelSyscalls {
    fn exit(exit_reason: ExitReason) -> ! {
        send_scheduler_event(crate::process::SchedulerEvent::Exit(exit_reason));
        unreachable!("Should nerver return");
    }

    fn map_memory(
        location: MemoryLocation,
        protections: MemoryProtections,
        bytes: usize,
    ) -> ::core::result::Result<*mut u8, MapMemoryError> {
        todo!()
    }

    fn get_pid() -> usize {
        todo!()
    }

    fn debug_msg(msg: &str) -> ::core::result::Result<(), DebugMsgError> {
        ::lldebug::priv_print(lldebug::LogKind::Log, "userspace", format_args!("{}", msg));

        Ok(())
    }
}
