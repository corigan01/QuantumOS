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
    locks::interrupt_locks_held,
    registers::{ProcessContext, Segment},
};
use mem::addr::{KERNEL_ADDR_START, VirtAddr};
use quantum_portal::{WaitCondition, WaitSignal};

use crate::{
    context::{kernelspace_entry, userspace_entry},
    processor::{assert_critical, is_within_irq, notify_end_critical, set_current_thread_id},
};

use super::KernelStack;

/// A structure repr one process thread on the system.
#[derive(Debug, Clone)]
pub struct Thread {
    pub id: usize,
    pub ue_context: ProcessContext,
    pub kn_context: Option<ProcessContext>,
    pub kn_invalid: bool,
    pub wait_conds: Vec<WaitCondition>,
    pub wait_signals: Vec<WaitSignal>,
    pub kernel_stack: Option<KernelStack>,
}

impl Thread {
    pub const USERSPACE_STACK_SEGMENT: u16 = 4;
    pub const USERSPACE_CODE_SEGMENT: u16 = 5;
    pub const KERNEL_STACK_SEGMENT: u16 = 2;
    pub const KERNEL_CODE_SEGMENT: u16 = 1;
    pub const RFLAGS_DEFAULT: u64 = 0x200;

    /// Create a new empty thread
    pub const fn new(thread_id: usize) -> Self {
        Self {
            id: thread_id,
            ue_context: ProcessContext::new(),
            kn_context: None,
            wait_conds: Vec::new(),
            wait_signals: Vec::new(),
            kernel_stack: None,
            kn_invalid: false,
        }
    }

    /// Set the userspace context
    pub fn set_userspace_context(&mut self, context: ProcessContext) {
        assert_eq!(
            context.cs >> 3,
            Self::USERSPACE_CODE_SEGMENT as u64,
            "{:#x?}",
            context
        );
        assert_eq!(context.ss >> 3, Self::USERSPACE_STACK_SEGMENT as u64);
        assert!((context.rip as usize) < KERNEL_ADDR_START.addr());
        assert_ne!(context.rip, 0);

        self.ue_context = context;
    }

    /// Get the Thread ID of this thread.
    pub const fn get_tid(&self) -> usize {
        self.id
    }

    /// Set the kernel context
    pub fn set_kernel_context(&mut self, context: ProcessContext, stack: KernelStack) {
        assert_eq!(context.cs >> 3, Self::KERNEL_CODE_SEGMENT as u64);
        assert_eq!(context.ss >> 3, Self::KERNEL_STACK_SEGMENT as u64);
        assert!((context.rip as usize) >= KERNEL_ADDR_START.addr());
        assert!(!self.kn_invalid);

        self.kernel_stack = Some(stack);
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
    pub unsafe fn context_switch(&mut self, global_stack: &KernelStack) -> ! {
        assert_eq!(
            interrupt_locks_held(),
            0,
            "Cannot switch while locks are being held"
        );
        assert!(
            !is_within_irq(),
            "Must clear IRQ status before switching into a new context!"
        );

        assert_interrupts(false);
        assert_critical(true);

        // Set the thread id
        set_current_thread_id(self.id);

        // If we have `kernel_context` that must mean we switched out of the kernel, meaning we should
        // switch back into the kernel.
        //
        // We also need to delete this context since we will be switching back to it.
        if let Some(ref kernel_context) = self.kn_context {
            self.kn_invalid = true;

            // set this stack as the current stack
            unsafe {
                self.kernel_stack
                    .as_ref()
                    .expect("Kernel thread must have kernel stack")
                    .set_syscall_stack()
            };

            // Since interrupts are disabled, this is safe to do now.
            notify_end_critical();

            unsafe { kernelspace_entry(kernel_context as *const _) };
        }

        self.kn_invalid = false;

        // Release our kernel stack, and aquire the global_stack
        unsafe { global_stack.set_syscall_stack() };
        self.kn_context = None;
        self.kernel_stack = None;

        // Since interrupts are disabled, this is safe to do now.
        notify_end_critical();

        // We must've previously been in userspace, so lets switch back into it
        unsafe { userspace_entry(&raw const self.ue_context) };
    }
}
