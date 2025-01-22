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

use core::{
    arch::{asm, global_asm},
    cell::SyncUnsafeCell,
    sync::atomic::{AtomicPtr, Ordering},
};

/// CPUs context
#[repr(C)]
pub struct ProcessContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub cs: u64,
    pub ss: u64,
    pub rflag: u64,
    pub rip: u64,
    pub exception_code: u64,
    pub rsp: u64,
}

/// The current user's stack ptr
#[unsafe(no_mangle)]
static USER_STACK_PTR: AtomicPtr<u64> = AtomicPtr::new(0 as *mut u64);
/// The current user's kernel stack ptr
#[unsafe(no_mangle)]
static KERN_STACK_PTR: AtomicPtr<u64> = AtomicPtr::new(0 as *mut u64);

#[inline]
fn read_user_stack_ptr() -> *mut u64 {
    USER_STACK_PTR.load(Ordering::Relaxed)
}

#[inline]
fn read_kernel_stack_ptr() -> *mut u64 {
    KERN_STACK_PTR.load(Ordering::Relaxed)
}

/// This is the entry of the kernel once being called from a syscall.
#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_entry() {
    unsafe {
        let user_stack_ptr = read_user_stack_ptr();
        let kernel_stack_ptr = read_kernel_stack_ptr();

        asm!(
            "
                #  -- Save User's stack ptr, and restore our own

                mov [{user_rsp}], rsp
                mov rsp, [{kernel_rsp}]

                #  -- Start building the processes `ProcessContext`

                push [{user_rsp}]
                push 0                         # This isn't an ISR, so we can just store 0 into its 'exception_code'
                push rcx                       # rip is saved in ecx
                push r11                       # rFLAGS is saved in r11
                push 0x23
                push 0x1b

                push rax
                push rbx
                push rcx
                push rdx
                push rsi
                push rdi
                push rbp
                push r8
                push r9
                push r10
                push r11
                push r12
                push r13
                push r14
                push r15

                #  -- Call the 'syscall_entry' function

                mov r9, rax                    # We move rax (syscall number) into r9 (arg5 of callee)
                call syscall_entry

                #  -- Start restoring the processes `ProcessContext`
                
                pop r15
                pop r14
                pop r13
                pop r12
                pop r11
                pop r10
                pop r9
                pop r8
                pop rbp
                pop rdi
                pop rsi
                pop rdx
                pop rcx
                pop rbx
                pop rax

                sub rsp, 8                     # pop ss
                sub rsp, 8                     # pop rsp
                sub rsp, 8                     # pop rflags
                sub rsp, 8                     # pop rip

                #  -- Return back to userspace

                cli
                pop rsp
                sysretq
                
            ",
            user_rsp = in(reg) user_stack_ptr,
            kernel_rsp = in(reg) kernel_stack_ptr,
        );
    }
}

/// This is the entry for userspace
#[unsafe(no_mangle)]
pub unsafe extern "C" fn userspace_entry(context: *const ProcessContext) {
    unsafe {
        asm!(
            "
                cli

                #  -- Restore Registers

                mov r15, [rdi      ]
                mov r14, [rdi + 8  ]
                mov r13, [rdi + 16 ]
                mov r13, [rdi + 24 ]
                mov r11, [rdi + 32 ]
                mov r10, [rdi + 40 ]
                mov r9,  [rdi + 48 ]
                mov r8,  [rdi + 56 ]
                mov rbp, [rdi + 64 ]
                mov rsi, [rdi + 80 ]
                mov rdx, [rdi + 88 ]
                mov rcx, [rdi + 96 ]
                mov rbx, [rdi + 104]
                mov rax, [rdi + 112]

                #  -- Restore TRAP frame

                push [rdi + 152] # errno
                push [rdi + 128] # ss
                push [rdi + 160] # rsp
                push [rdi + 136] # rflags
                push [rdi + 120] # cs
                push [rdi + 144] # rip

                push [rdi + 72 ]
                pop rdi
                
                #  -- Return back to userspace

                iretq

            ",
            in("rdi") context
        )
    }
}
