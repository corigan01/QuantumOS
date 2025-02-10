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

use crate::process::{SchedulerEvent, aquire_running_and_release_biglock, send_scheduler_event};
use alloc::format;
use mem::paging::VmPermissions;
use quantum_portal::{
    DebugMsgError, ExitReason, MapMemoryError, MemoryLocation, MemoryProtections, QuantumPortal,
    WaitCondition, WaitSignal, WaitingError, server::QuantumPortalServer,
};
use util::consts::PAGE_4K;

pub struct KernelSyscalls {}

impl QuantumPortalServer for KernelSyscalls {
    fn verify_user_ptr<T: Sized>(_ptr: *const T) -> bool {
        // TODO: Add ptr verify
        true
    }
}

impl QuantumPortal for KernelSyscalls {
    fn exit(exit_reason: ExitReason) -> ! {
        send_scheduler_event(SchedulerEvent::Exit(exit_reason));
        unreachable!("Should nerver return");
    }

    fn map_memory(
        location: MemoryLocation,
        protections: MemoryProtections,
        bytes: usize,
    ) -> ::core::result::Result<*mut u8, MapMemoryError> {
        let process = aquire_running_and_release_biglock();
        let page_permissions = match protections {
            MemoryProtections::ReadOnly => VmPermissions::USER_R,
            MemoryProtections::ReadWrite => VmPermissions::USER_RW,
            MemoryProtections::ReadExecute => VmPermissions::USER_RE,
            MemoryProtections::None => VmPermissions::none(),
        };
        let page_amount = ((bytes - 1) / PAGE_4K) + 1;

        // FIXME: We need to figure out a sane number of max pages.
        if page_amount > 1024 || bytes == 0 {
            return Err(MapMemoryError::InvalidLength(bytes));
        }

        let vm_region = match location {
            MemoryLocation::Anywhere => process
                .write()
                .add_anywhere(page_amount, page_permissions, false)
                .map_err(|err| match err {
                    crate::process::ProcessError::OutOfVirtualMemory => MapMemoryError::OutOfMemory,
                    _ => MapMemoryError::MappingMemoryError,
                })?,
        };

        Ok(vm_region.start.addr().as_mut_ptr())
    }

    fn get_pid() -> usize {
        let process = aquire_running_and_release_biglock();
        process.read().get_pid()
    }

    fn debug_msg(msg: &str) -> ::core::result::Result<(), DebugMsgError> {
        let process = aquire_running_and_release_biglock();

        let fmt_string = format!(
            "{:<22}pid={:04}",
            process.read().get_process_name(),
            process.read().get_pid(),
        );

        ::lldebug::priv_print(lldebug::LogKind::Log, &fmt_string, format_args!("{}", msg));

        Ok(())
    }

    fn wait_for(
        conditions: &[WaitCondition],
        signal_buffer: &mut [WaitSignal],
    ) -> ::core::result::Result<usize, WaitingError> {
        todo!()
    }
}
