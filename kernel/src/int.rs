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

use crate::{
    process::{
        AccessViolationReason, Process, ProcessError,
        scheduler::{get_running, scheduler_process_crash},
    },
    processor::{notify_begin_irq, notify_end_irq},
};
use arch::{
    CpuPrivilege, attach_irq, critcal_section,
    idt64::{
        ExceptionKind, InterruptDescTable, InterruptFlags, InterruptInfo, fire_debug_int, interrupt,
    },
    interrupts::{self, assert_interrupts},
    locks::{InterruptMutex, interrupt_locks_held},
    pic8259::{pic_eoi, pic_remap},
    registers::{ProcessContext, Segment},
};
use lldebug::{log, logln};
use mem::{
    addr::VirtAddr,
    vm::{PageFaultInfo, call_page_fault_handler},
};
use quantum_portal::server::QuantumPortalServer;

static INTERRUPT_TABLE: InterruptMutex<InterruptDescTable> =
    InterruptMutex::new(InterruptDescTable::new());
static IRQ_HANDLERS: InterruptMutex<[Option<fn(&InterruptInfo)>; 32]> =
    InterruptMutex::new([None; 32]);

#[interrupt(0..50)]
fn exception_handler(args: &InterruptInfo) {
    assert_eq!(
        interrupt_locks_held(),
        0,
        "Should not be possible to call an interrupt while interrupt locks are held -- {args:#?}"
    );
    assert_interrupts(false);
    notify_begin_irq();
    let rsp: u64;
    unsafe { core::arch::asm!("mov rax, rsp", out("rax") rsp) };
    let rsp = VirtAddr::new(rsp as usize);

    logln!("rsp={:#018x} - {:?}", rsp, args.flags);

    match args.flags {
        // IRQ
        InterruptFlags::Irq(irq_num) if irq_num - PIC_IRQ_OFFSET <= 16 => {
            unsafe { pic_eoi(irq_num - PIC_IRQ_OFFSET) };
            call_attached_irq(irq_num - PIC_IRQ_OFFSET, &args);
        }
        InterruptFlags::PageFault {
            present,
            write,
            user,
            reserved_write,
            instruction_fetch,
            protection_key,
            shadow_stack,
            software_guard,
            virt_addr,
        } => {
            let vaddr = VirtAddr::new(virt_addr as usize);
            let info = PageFaultInfo {
                is_present: present,
                write_read_access: write,
                execute_fault: instruction_fetch,
                user_fault: user,
                vaddr,
            };
            match call_page_fault_handler(info) {
                // If this page fault was handled, we dont need to do anything!
                mem::vm::PageFaultReponse::Handled => (),
                // Crash the process
                mem::vm::PageFaultReponse::NoAccess {
                    page_perm,
                    request_perm,
                    addr,
                } => {
                    logln!("{:#x?}", args);
                    let (proc, _) = get_running();
                    scheduler_process_crash(
                        proc,
                        ProcessError::AccessViolation(AccessViolationReason::NoAccess {
                            page_perm,
                            request_perm,
                            addr,
                        }),
                    );
                }
                // panic
                mem::vm::PageFaultReponse::CriticalFault(error) => {
                    panic!("PageFault critical fault: {error}");
                }
                // panic
                mem::vm::PageFaultReponse::NotAttachedHandler => {
                    panic!(
                        "PageFault without attached handler!\n{:#016x?}\n{:#016x?}",
                        info, args
                    );
                }
            }
        }
        InterruptFlags::Debug => {
            lldebug::logln!("{:#x?}", args);
        }
        exception => {
            panic!("UNHANDLED FAULT\n{:#016x?}", args)
        }
        _ => (),
    }

    if args.flags.exception_kind() == ExceptionKind::Abort {
        panic!("Interrupt -- {:?}", args.flags);
    }

    notify_end_irq();
    assert_interrupts(false);
}

fn call_attached_irq(irq_id: u8, args: &InterruptInfo) {
    let irq_handler = IRQ_HANDLERS.lock();

    if let Some(handler) = irq_handler
        .get((irq_id) as usize)
        .and_then(|&handler| handler)
    {
        // Release the lock on the irq handler since we
        // don't know if the handler is ever going to return!
        drop(irq_handler);

        // Finally call the handler
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

/// Attach the main 'exception handler' function to the IDT
pub fn attach_interrupts() {
    {
        let mut idt = INTERRUPT_TABLE.lock();
        attach_irq!(idt, exception_handler);
        unsafe { idt.submit_table().load() };

        logln!("Attached Interrupts!");
    }

    log!("Checking Interrupts...");
    fire_debug_int();
    logln!("OK");
}

/// Attach the main 'syscall' entrypoint handler to the IDT
pub fn attach_syscall() {
    logln!("Attaching Syscalls...");

    // Now attach the 'SYSCALL' instruction handler
    unsafe {
        arch::registers::ia32_efer::set_syscall_extensions_flag(true);
        arch::registers::amd_syscall::set_syscall_target_ptr(crate::context::kernel_entry as u64);
        arch::registers::amd_syscall::set_rflags_mask(0x257fd5);
        arch::registers::amd_syscall::write_kernel_segments(
            Segment::new(1, CpuPrivilege::Ring0),
            Segment::new(2, CpuPrivilege::Ring0),
        );
        arch::registers::amd_syscall::write_userspace_segments(
            Segment::new(5, CpuPrivilege::Ring3),
            Segment::new(4, CpuPrivilege::Ring3),
        );
    }

    logln!("Syscall instruction attached!");
}

const PIC_IRQ_OFFSET: u8 = 0x20;

/// Enable the PIC
pub fn enable_pic() {
    unsafe {
        pic_remap(PIC_IRQ_OFFSET, PIC_IRQ_OFFSET + 8);
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
extern "C" fn syscall_handler(
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rsp: u64,
    r8: u64,
    syscall_number: u64,
) -> u64 {
    unsafe {
        let b4 = {
            logln!("SYSCALL");
            let (_, thread) = get_running();
            thread
                .lock()
                .set_userspace_context(core::ptr::read_volatile(rsp as *const ProcessContext));
            core::ptr::read_volatile(rsp as *const ProcessContext)
        };

        // Call the portal
        let resp =
            crate::syscall_handler::KernelSyscalls::from_syscall(syscall_number, rdi, rsi, rdx, r8);

        interrupts::disable_interrupts();

        {
            let (_, thread) = get_running();
            let c = thread.lock().ue_context;
            logln!("S{:#x?} {}", c, c == b4);
        }

        resp
    }
}
