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

use core::arch::global_asm;

use arch::{
    attach_irq, critcal_section,
    idt64::{
        ExceptionKind, GateDescriptor, InterruptDescTable, InterruptFlags, InterruptInfo,
        fire_debug_int, interrupt,
    },
    interrupts::enable_interrupts,
    pic8259::{pic_eoi, pic_remap},
    registers::Segment,
};
use lldebug::{log, logln, sync::Mutex};

static INTERRUPT_TABLE: Mutex<InterruptDescTable> = Mutex::new(InterruptDescTable::new());
static IRQ_HANDLERS: Mutex<[Option<fn(&InterruptInfo)>; 32]> = Mutex::new([None; 32]);

#[interrupt(0..48)]
fn main_handler(args: InterruptInfo) {
    match args.flags {
        // IRQ
        InterruptFlags::Irq(irq_num) if irq_num - PIC_IRQ_OFFSET <= 16 => {
            call_attached_irq(irq_num - PIC_IRQ_OFFSET, &args);
            unsafe { pic_eoi(irq_num - PIC_IRQ_OFFSET) };
        }
        InterruptFlags::Debug => (),
        exception => {
            panic!("UNHANDLED FAULT\n{:#016x?}", args)
        }
        _ => (),
    }

    if args.flags.exception_kind() == ExceptionKind::Abort {
        panic!("Interrupt -- {:?}", args.flags);
    }
}

fn call_attached_irq(irq_id: u8, args: &InterruptInfo) {
    let irq_handler = IRQ_HANDLERS.lock();

    if let Some(handler) = irq_handler
        .get((irq_id) as usize)
        .and_then(|&handler| handler)
    {
        handler(args);
    }
}

/// Set a function to be called whenever an irq is triggered.
pub fn attach_irq_handler(handler_fn: fn(&InterruptInfo), irq: u8) {
    critcal_section! {
        let mut irq_handler = IRQ_HANDLERS.lock();
        let Some(handler) = irq_handler.get_mut(irq as usize) else {
            return;
        };

        *handler = Some(handler_fn);
    }
}

pub fn attach_interrupts() {
    let mut idt = INTERRUPT_TABLE.lock();
    attach_irq!(idt, main_handler);
    unsafe { idt.submit_table().load() };

    logln!("Attached Interrupts!");

    log!("Checking Interrupts...");
    fire_debug_int();
    logln!("OK");
}

pub fn attach_syscall() {
    let mut idt = INTERRUPT_TABLE.lock();
    logln!("Attaching Syscalls...");

    let mut gate = GateDescriptor::zero();
    gate.set_present_flag(true);
    gate.set_privilege(arch::CpuPrivilege::Ring3);
    gate.set_offset(syscall_entry as u64);
    gate.set_gate_kind(arch::idt64::GateKind::InterruptGate);
    gate.set_code_segment(Segment::new(3, arch::CpuPrivilege::Ring3));

    idt.attach_raw(0x80, gate);
    unsafe { idt.submit_table().load() };
    logln!("Syscall attached!");
}

const PIC_IRQ_OFFSET: u8 = 0x20;

pub fn enable_pic() {
    unsafe {
        pic_remap(PIC_IRQ_OFFSET, PIC_IRQ_OFFSET + 8);
        enable_interrupts();
    }
}

unsafe extern "C" {
    fn syscall_entry();
    pub fn task_start();
}

#[unsafe(no_mangle)]
extern "C" fn syscall_handler() {
    todo!()
}

global_asm!(
    r#"
    syscall_entry:
        cli
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
        call syscall_handler
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
        iretq"#
);

global_asm!(
    r#"
    .global task_start
    task_start:
        cli
        # Save our state to the stack
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

        #errno
        push 0
        # interrupt stack
        push 0x23
        mov rax, 0x200000000000 
        push rax
        # rflags
        push 0x200
        # Code segment
        push 0x1b

        # Task Entry Point
        mov rax, 0x00000000200000
        push rax

        # Init UE with zeroed registers
        xor rax, rax
        xor rbx, rbx
        xor rcx, rcx
        xor rdx, rdx
        xor rbp, rbp
        xor r8, r8
        xor r9, r9
        xor r10, r10
        xor r11, r11
        xor r12, r12
        xor r13, r13
        xor r14, r14
        xor r15, r15
        iretq
    "#
);
