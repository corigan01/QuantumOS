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

use alloc::vec::Vec;
use arch::{
    CpuPrivilege,
    interrupts::assert_interrupts,
    registers::{ProcessContext, Segment},
};
use mem::addr::VirtAddr;
use quantum_portal::{WaitCondition, WaitSignal};

use crate::{
    context::userspace_entry,
    processor::{
        assert_critical, get_current_thread_id, notify_end_critical, set_current_thread_id,
    },
};

/// A structure repr one process thread on the system.
#[derive(Debug, Clone)]
pub struct Thread {
    id: usize,
    ue_context: ProcessContext,
    kn_context: Option<ProcessContext>,
    wait_conds: Vec<WaitCondition>,
    wait_signals: Vec<WaitSignal>,
}

impl Thread {
    pub const USERSPACE_STACK_SEGMENT: u16 = 4;
    pub const USERSPACE_CODE_SEGMENT: u16 = 5;
    pub const RFLAGS_DEFAULT: u64 = 0x200;

    /// Create a new empty thread
    pub const fn new(thread_id: usize) -> Self {
        Self {
            id: thread_id,
            ue_context: ProcessContext::new(),
            kn_context: None,
            wait_conds: Vec::new(),
            wait_signals: Vec::new(),
        }
    }

    /// Set the userspace context
    pub const fn set_userspace_context(&mut self, context: ProcessContext) {
        self.ue_context = context;
    }

    /// Get the Thread ID of this thread.
    pub const fn get_tid(&self) -> usize {
        self.id
    }

    /// Set the kernel context
    pub const fn set_kernel_context(&mut self, context: ProcessContext) {
        self.kn_context = Some(context);
    }

    /// Init the thread's context structure
    pub fn init_user_context(&mut self, entry: VirtAddr, stack: VirtAddr) {
        self.ue_context = ProcessContext::new();
        self.ue_context.ss =
            Segment::new(Self::USERSPACE_STACK_SEGMENT, CpuPrivilege::Ring3).0 as u64;
        self.ue_context.cs =
            Segment::new(Self::USERSPACE_CODE_SEGMENT, CpuPrivilege::Ring3).0 as u64;
        self.ue_context.rip = entry.addr() as u64;
        self.ue_context.rsp = stack.addr() as u64;
        self.ue_context.rflag = Self::RFLAGS_DEFAULT;
    }

    /// Switch into a new context.
    ///
    /// # Note
    /// This will switch into either kernel/userspace, whichever was last running. For example, if userspace
    /// had invoked a syscall, but the process's quantum expired, the next time this thread should be
    /// scheduled will return into the kernel.
    ///
    /// # Safety
    /// The process context should always point to valid memory.
    pub unsafe fn context_switch(&mut self) -> ! {
        assert_interrupts(false);
        assert_critical(true);

        // Check if we are currently in our own thread
        let current_thread_id = get_current_thread_id();
        if current_thread_id == self.id {
            assert!(
                self.kn_context.is_none(),
                "Cannot switch into own kernel context"
            );
        } else {
            set_current_thread_id(self.id);
        }

        // Since interrupts are disabled, this is safe to do now.
        notify_end_critical();

        // If we have `kernel_context` that must mean we switched out of the kernel, meaning we should
        // switch back into the kernel.
        //
        // We also need to delete this context since we will be switching back to it.
        if let Some(kernel_context) = self.kn_context {
            self.kn_context = None;

            // FIXME: we should call this something else, because here we are just switching back into
            // the kernel.
            unsafe { userspace_entry(&raw const kernel_context) };
            unreachable!("Kernel should never return back to `context_switch`")
        }

        // We must've previously been in userspace, so lets switch back into it
        unsafe { userspace_entry(&raw const self.ue_context) };
        unreachable!("Userspace should never return back to `context_switch`");
    }
}
