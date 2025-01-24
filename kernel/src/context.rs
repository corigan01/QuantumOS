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

/// CPUs context
#[repr(C)]
#[derive(Clone, Copy, Debug)]
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

pub static mut KERNEL_RSP_PTR: u64 = 0x200000000000;
pub static mut USERSPACE_RSP_PTR: u64 = 0x121212;

#[naked]
pub unsafe extern "C" fn kernel_entry() {
    unsafe {
        naked_asm!(
            "
            #  -- Save User's stack ptr, and restore our own

            mov [{userspace_rsp_ptr}], rsp
            mov rsp, [{kernel_rsp_ptr}]

            #  -- Start building the processes `ProcessContext`

            push [{userspace_rsp_ptr}]
            sti

            push 0                         # This isn't an ISR, so we can just store 0 into its 'exception_code'
            push rcx                       # rip is saved in rcx
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

            sysretq
        ",
            // FIXME: For whatever reason, rust fails to compile with these symbol PTRs,
            //        so for right now I will just make them part of the linker script and
            //        use raw ptrs to these symbols.
            // kernel_rsp_ptr = sym KERNEL_RSP,
            // userspace_rsp_ptr = sym USERSPACE_RSP,

            kernel_rsp_ptr = sym KERNEL_RSP_PTR ,
            userspace_rsp_ptr = sym USERSPACE_RSP_PTR ,
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
    unreachable!("Should never return from userspace entry!");
}

pub unsafe fn context_of_caller() -> ProcessContext {
    let mut pc = ProcessContext {
        r15: 0,
        r14: 0,
        r13: 0,
        r12: 0,
        r11: 0,
        r10: 0,
        r9: 0,
        r8: 0,
        rbp: 0,
        rdi: 0,
        rsi: 0,
        rdx: 0,
        rcx: 0,
        rbx: 0,
        rax: 0,
        cs: 0,
        ss: 0,
        rflag: 0,
        rip: 0,
        exception_code: 0,
        rsp: 0,
    };

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
