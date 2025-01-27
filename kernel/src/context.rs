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
    arch::{asm, naked_asm},
    mem::offset_of,
};

use crate::process::Process;
use arch::registers::ProcessContext;

/// The kernel's syscall entry stack
pub static mut KERNEL_RSP_PTR: u64 = Process::KERNEL_SYSCALL_STACK_ADDR.addr() as u64;
/// A tmp for userspace's stack ptr while in kernel land
pub static mut USERSPACE_RSP_PTR: u64 = 0x121212;
/// A lock for timer ticks to not attempt to lock the scheduler
pub static mut IN_USERSPACE: u8 = 0;

#[naked]
pub unsafe extern "C" fn kernel_entry() {
    unsafe {
        naked_asm!(
            "
            #  -- Save User's stack ptr, and restore our own

            mov byte ptr [{user_lock}], 0
            mov [{userspace_rsp_ptr}], rsp
            mov rsp, [{kernel_rsp_ptr}]

            #  -- Start building the processes `ProcessContext`

            sti

            push 0x1b
            push [{userspace_rsp_ptr}]
            push r11                       # rFLAGS is saved in r11
            push 0x23
            push rcx                       # rip is saved in rcx
            push 0                         # This isn't an ISR, so we can just store 0 into its 'exception_code'

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
            call syscall_handler

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

            add rsp, 8                     # pop cs
            add rsp, 8                     # pop ss
            add rsp, 8                     # pop r11
            add rsp, 8                     # pop rcx
            add rsp, 8                     # pop 0

            #  -- Return back to userspace

            cli
            pop rsp

            mov qword ptr [{userspace_rsp_ptr}], 0
            mov byte ptr [{user_lock}], 1
            sysretq
        ",
            // FIXME: For whatever reason, rust fails to compile with these symbol PTRs,
            //        so for right now I will just make them part of the linker script and
            //        use raw ptrs to these symbols.
            // kernel_rsp_ptr = sym KERNEL_RSP,
            // userspace_rsp_ptr = sym USERSPACE_RSP,

            kernel_rsp_ptr = sym KERNEL_RSP_PTR,
            userspace_rsp_ptr = sym USERSPACE_RSP_PTR,
            user_lock = sym IN_USERSPACE,
        )
    };
}

/// This is the entry for userspace
#[unsafe(no_mangle)]
pub unsafe extern "C" fn userspace_entry(context: *const ProcessContext) {
    unsafe {
        asm!(
            "
                cli

                #  -- Restore Registers

                mov r15, [rdi + {r15_offset} ]
                mov r14, [rdi + {r14_offset} ]
                mov r13, [rdi + {r13_offset} ]
                mov r13, [rdi + {r13_offset} ]
                mov r12, [rdi + {r12_offset} ]
                mov r11, [rdi + {r11_offset} ]
                mov r10, [rdi + {r10_offset} ]
                mov r9,  [rdi + {r9_offset}  ]
                mov r8,  [rdi + {r8_offset}  ]
                mov rbp, [rdi + {rbp_offset} ]
                mov rsi, [rdi + {rsi_offset} ]
                mov rdx, [rdi + {rdx_offset} ]
                mov rcx, [rdi + {rcx_offset} ]
                mov rbx, [rdi + {rbx_offset} ]
                mov rax, [rdi + {rax_offset} ]

                #  -- Restore TRAP frame

                push [rdi + {errno_offset}   ] # errno
                push [rdi + {ss_offset}      ] # ss
                push [rdi + {rsp_offset}     ] # rsp
                push [rdi + {rflags_offset}  ] # rflags
                push [rdi + {cs_offset}      ] # cs
                push [rdi + {rip_offset}     ] # rip

                push [rdi + {rdi_offset}     ] # rdi
                pop rdi

                #  -- Return back to userspace

                mov byte ptr [{user_lock}], 1
                iretq
                ",
            in("rdi") context,
            user_lock = sym IN_USERSPACE,
            rip_offset = const { offset_of!(ProcessContext, rip) },
            r15_offset = const { offset_of!(ProcessContext, r15) },
            r14_offset = const { offset_of!(ProcessContext, r14) },
            r13_offset = const { offset_of!(ProcessContext, r13) },
            r12_offset = const { offset_of!(ProcessContext, r12) },
            r11_offset = const { offset_of!(ProcessContext, r11) },
            r10_offset = const { offset_of!(ProcessContext, r10) },
            r9_offset = const { offset_of!(ProcessContext, r9) },
            r8_offset = const { offset_of!(ProcessContext, r8) },
            rbp_offset = const { offset_of!(ProcessContext, rbp ) },
            rdi_offset = const { offset_of!(ProcessContext, rdi ) },
            rsi_offset = const { offset_of!(ProcessContext, rsi ) },
            rdx_offset = const { offset_of!(ProcessContext, rdx ) },
            rcx_offset = const { offset_of!(ProcessContext, rcx ) },
            rbx_offset = const { offset_of!(ProcessContext, rbx ) },
            rax_offset = const { offset_of!(ProcessContext, rax ) },
            cs_offset = const { offset_of!(ProcessContext, cs ) },
            ss_offset = const { offset_of!(ProcessContext, ss ) },
            rflags_offset = const { offset_of!(ProcessContext, rflag ) },
            rsp_offset = const { offset_of!(ProcessContext, rsp ) },
            errno_offset = const { offset_of!(ProcessContext,  exception_code) },

        )
    }
    unreachable!("Should never return from userspace entry!");
}

pub unsafe fn context_of_caller() -> ProcessContext {
    let mut pc = ProcessContext::new();
    unsafe {
        asm!(
            "
                cli
                mov [{pc_ptr} + {r15}], r15 
                mov [{pc_ptr} + {r14}], r14 
                mov [{pc_ptr} + {r13}], r13 
                mov [{pc_ptr} + {r12}], r12 
                mov [{pc_ptr} + {r11}], r11 
                mov [{pc_ptr} + {r10}], r10 
                mov [{pc_ptr} + {r9}], r9 
                mov [{pc_ptr} + {r8}], r8 
                mov [{pc_ptr} + {rbp}], rbp 
                mov [{pc_ptr} + {rdi}], rdi 
                mov [{pc_ptr} + {rsi}], rsi 
                mov [{pc_ptr} + {rdx}], rdx 
                mov [{pc_ptr} + {rcx}], rcx 
                mov [{pc_ptr} + {rbx}], rbx 
                mov [{pc_ptr} + {rax}], rax 
                mov [{pc_ptr} + {cs}], cs 
                mov [{pc_ptr} + {ss}], ss 
                pushf
                pop rax
                mov [{pc_ptr} + {rflags}], rax
                mov [{pc_ptr} + {rsp}], rsp 
                sti
            ",
          pc_ptr = in(reg) &raw mut pc,
          out("rax") _ ,
          r15 = const { offset_of!(ProcessContext, r15) },
          r14 = const { offset_of!(ProcessContext, r14) },
          r13 = const { offset_of!(ProcessContext, r13) },
          r12 = const { offset_of!(ProcessContext, r12) },
          r11 = const { offset_of!(ProcessContext, r11) },
          r10 = const { offset_of!(ProcessContext, r10) },
          r9 = const { offset_of!(ProcessContext, r9) },
          r8 = const { offset_of!(ProcessContext, r8) },
          rbp = const { offset_of!(ProcessContext, rbp ) },
          rdi = const { offset_of!(ProcessContext, rdi ) },
          rsi = const { offset_of!(ProcessContext, rsi ) },
          rdx = const { offset_of!(ProcessContext, rdx ) },
          rcx = const { offset_of!(ProcessContext, rcx ) },
          rbx = const { offset_of!(ProcessContext, rbx ) },
          rax = const { offset_of!(ProcessContext, rax ) },
          cs = const { offset_of!(ProcessContext, cs ) },
          ss = const { offset_of!(ProcessContext, ss ) },
          rflags = const { offset_of!(ProcessContext, rflag ) },
          rsp = const { offset_of!(ProcessContext, rsp ) },
        );
    }

    pc
}
