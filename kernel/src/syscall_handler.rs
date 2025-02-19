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

use alloc::format;
use mem::paging::VmPermissions;
use quantum_portal::{
    DebugMsgError, ExitReason, MapMemoryError, MemoryLocation, MemoryProtections, QuantumPortal,
    WaitCondition, WaitSignal, WaitingError, server::QuantumPortalServer,
};
use util::consts::PAGE_4K;

use crate::process::scheduler::{get_running_and_release_lock, scheduler_exit_process};

pub struct KernelSyscalls {}

impl QuantumPortalServer for KernelSyscalls {
    fn verify_user_ptr<T: Sized>(ptr: *const T) -> bool {
        let (proc, _) = get_running_and_release_lock();

        unsafe { &*proc.as_mut_ptr() }
            .check_addr(ptr.into(), VmPermissions::USER_RW)
            .is_ok()
    }
}

impl QuantumPortal for KernelSyscalls {
    fn exit(_exit_reason: ExitReason) -> ! {
        let (process, _) = get_running_and_release_lock();
        scheduler_exit_process(process);
    }

    fn map_memory(
        location: MemoryLocation,
        protections: MemoryProtections,
        bytes: usize,
    ) -> Result<*mut u8, MapMemoryError> {
        let (process, _) = get_running_and_release_lock();
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
                .lock()
                .add_anywhere(page_amount, page_permissions, false)
                .map_err(|err| match err {
                    crate::process::ProcessError::OutOfVirtualMemory => MapMemoryError::OutOfMemory,
                    _ => MapMemoryError::MappingMemoryError,
                })?,
        };

        Ok(vm_region.start.addr().as_mut_ptr())
    }

    fn get_pid() -> usize {
        let (process, _) = get_running_and_release_lock();
        process.lock().get_pid()
    }

    fn debug_msg(msg: &str) -> Result<(), DebugMsgError> {
        let (process, thread) = get_running_and_release_lock();

        let fmt_string = {
            let proc_lock = process.lock();
            let thread_lock = thread.lock();
            format!(
                "{:<23}t{:02x} p{:02x}",
                proc_lock.get_process_name(),
                thread_lock.get_tid(),
                proc_lock.get_pid(),
            )
        };

        ::lldebug::priv_print(lldebug::LogKind::Log, &fmt_string, format_args!("{}", msg));

        Ok(())
    }

    fn wait_for(
        conditions: &[WaitCondition],
        signal_buffer: &mut [WaitSignal],
    ) -> Result<usize, WaitingError> {
        let (_, _) = get_running_and_release_lock();

        // logln!(
        //     "Waiting - {:?} (signal return size = {})",
        //     conditions,
        //     signal_buffer.len()
        // );

        if signal_buffer.len() == 0 {
            return Err(WaitingError::InvalidSignalBuffer);
        }

        loop {}
        Ok(1)
    }
}
