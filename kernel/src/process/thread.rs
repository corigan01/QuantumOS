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

use core::arch::asm;

use super::{ProcessEntry, RefProcess, scheduler::Scheduler, task::Task};
use crate::{context::set_syscall_rsp, gdt, locks::ThreadCell};
use alloc::sync::{Arc, Weak};
use arch::interrupts;
use lldebug::logln;
use mem::{addr::VirtAddr, paging::VmPermissions, vm::VmRegion};
use util::consts::PAGE_4K;

pub type UserspaceStackTop = VirtAddr;
pub type ThreadId = usize;
pub type RefThread = Arc<Thread>;
pub type WeakThread = Weak<Thread>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadContextKind {
    Userspace,
    Kernel,
}

/// A userspace execution unit, like a [`Task`] but for userspace.
#[derive(Debug)]
pub struct Thread {
    /// The thread's id
    pub id: ThreadId,
    /// To which does this thread switches to, Kernel or Userspace
    pub context_kind: ThreadContextKind,
    /// This is the kernel task that starts/resumes the context.
    ///
    /// The context itself is stored within the task's stack, and could be
    /// placed either via an interrupt or via a system call.
    pub task: ThreadCell<Task>,
    /// The parent process that this thread represents
    pub process: RefProcess,
    /// Init Userspace entrypoint
    // TODO: Maybe there could be a better way of passing the `ProcessEntry` into
    // `userspace_thread_begin`?
    userspace_entry_ptr: Option<ProcessEntry>,
    userspace_rsp_ptr: ThreadCell<Option<UserspaceStackTop>>,
    pub crashed: ThreadCell<bool>,
}

impl Thread {
    pub const DEFAULT_USERSPACE_RSP_TOP: VirtAddr = VirtAddr::new(0x7fff00000000);
    pub const DEFAULT_USERSPACE_RSP_LEN: usize = PAGE_4K * 16;

    /// Create a new userspace thread
    pub fn new_user(process: RefProcess, entry_point: ProcessEntry) -> RefThread {
        let id = process.alloc_thread_id();
        let task = Task::new(userspace_thread_begin);

        let thread = Arc::new(Self {
            id,
            context_kind: ThreadContextKind::Userspace,
            task: ThreadCell::new(task),
            process,
            userspace_entry_ptr: Some(entry_point),
            userspace_rsp_ptr: ThreadCell::new(None),
            crashed: ThreadCell::new(false),
        });

        let s = Scheduler::get();
        s.register_new_thread(thread.clone());
        thread.alloc_user_stack();

        thread
    }

    /// Create a new kernel thread
    pub fn new_kernel(process: RefProcess, entry_point: fn()) -> RefThread {
        let id = process.alloc_thread_id();
        let task = Task::new(entry_point);

        let thread = Arc::new(Self {
            id,
            context_kind: ThreadContextKind::Kernel,
            task: ThreadCell::new(task),
            process,
            userspace_entry_ptr: None,
            userspace_rsp_ptr: ThreadCell::new(None),
            crashed: ThreadCell::new(false),
        });

        let s = Scheduler::get();
        s.register_new_thread(thread.clone());

        thread
    }

    /// Create a mapping for the userspace stack
    fn alloc_user_stack(&self) {
        let stack_top = Self::DEFAULT_USERSPACE_RSP_TOP
            .offset(self.id * Self::DEFAULT_USERSPACE_RSP_LEN + (self.id * PAGE_4K));

        self.process.map_anon(
            VmRegion::from_containing(
                stack_top.sub_offset(Self::DEFAULT_USERSPACE_RSP_LEN),
                stack_top,
            ),
            VmPermissions::USER_RW,
        );

        *self.userspace_rsp_ptr.borrow_mut() = Some(stack_top);
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        logln!("DROP");
    }
}

fn userspace_thread_begin() {
    let current_thread = Scheduler::get()
        .current_thread()
        .upgrade()
        .expect("Expected to operating on an alive thread");
    assert!(matches!(
        current_thread.context_kind,
        ThreadContextKind::Userspace
    ));

    let entry = current_thread
        .userspace_entry_ptr
        .expect("Requires an entry ptr")
        .addr();
    let rsp = current_thread
        .userspace_rsp_ptr
        .clone()
        .into_inner()
        .expect("Requires an rsp ptr")
        .addr();

    // Here we need a critical section because we need to ensure the ISR stack is set
    unsafe { interrupts::disable_interrupts() };

    let top_of_task_stack = current_thread.task.borrow().stack_top();
    gdt::set_stack_for_privl(top_of_task_stack.as_mut_ptr(), arch::CpuPrivilege::Ring0);
    unsafe { set_syscall_rsp(top_of_task_stack.addr() as u64) };

    unsafe {
        asm!(
            r"
              mov r15, 0
              mov r14, 0
              mov r13, 0
              mov r13, 0
              mov r12, 0
              mov r11, 0
              mov r10, 0
              mov r9,  0
              mov r8,  0
              mov rbp, 0
              mov rdx, 0
              mov rcx, 0
              mov rbx, 0
              mov rax, 0

              push 0x23   # ss
              push rsi    # rsp
              push 0x200  # rflags
              push 0x2b   # cs
              push rdi    # rip

              mov rsi, 0
              mov rdi, 0

              iretq
          ",
          in("rdi") entry,
          in("rsi") rsp,
        );
    }
}
