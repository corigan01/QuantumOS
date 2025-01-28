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

use core::sync::atomic::{AtomicBool, Ordering};

use arch::{
    CpuPrivilege, attach_irq, critcal_section,
    idt64::{
        ExceptionKind, InterruptDescTable, InterruptFlags, InterruptInfo, fire_debug_int, interrupt,
    },
    interrupts::enable_interrupts,
    pic8259::{pic_eoi, pic_remap},
    registers::Segment,
};
use lldebug::{log, logln, sync::Mutex, warnln};
use mem::{
    addr::VirtAddr,
    paging::VmPermissions,
    vm::{PageFaultInfo, call_page_fault_handler},
};
use util::consts::PAGE_4K;

use crate::{
    context::IN_USERSPACE,
    process::{
        AccessViolationReason, ProcessError, SchedulerEvent, lock_ref_scheduler,
        send_scheduler_event,
    },
};

static INTERRUPT_TABLE: Mutex<InterruptDescTable> = Mutex::new(InterruptDescTable::new());
static IRQ_HANDLERS: Mutex<[Option<fn(&InterruptInfo, bool)>; 32]> = Mutex::new([None; 32]);
static SHOULD_INTERRUPTS_BE_ENABLED: AtomicBool = AtomicBool::new(false);

#[interrupt(0..50)]
fn exception_handler(args: &InterruptInfo) {
    let called_from_ue = unsafe { core::ptr::read_volatile(&raw const IN_USERSPACE) };
    unsafe { core::ptr::write_volatile(&raw mut IN_USERSPACE, 0) };

    match args.flags {
        // IRQ
        InterruptFlags::Irq(irq_num) if irq_num - PIC_IRQ_OFFSET <= 16 => {
            call_attached_irq(irq_num - PIC_IRQ_OFFSET, &args, called_from_ue == 1);
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
                    page,
                } => {
                    warnln!("Process crash!");
                    send_scheduler_event(SchedulerEvent::Fault(ProcessError::AccessViolation(
                        AccessViolationReason::NoAccess {
                            page_perm,
                            request_perm,
                            page,
                        },
                    )));
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
        InterruptFlags::Debug => (),
        exception => {
            panic!("UNHANDLED FAULT\n{:#016x?}", args)
        }
        _ => (),
    }

    if args.flags.exception_kind() == ExceptionKind::Abort {
        panic!("Interrupt -- {:?}", args.flags);
    }

    if SHOULD_INTERRUPTS_BE_ENABLED.load(Ordering::Relaxed) {
        unsafe { enable_interrupts() };
    }

    // Send End of interrupt
    match args.flags {
        InterruptFlags::Irq(irq) if irq - PIC_IRQ_OFFSET <= 16 => {
            unsafe { pic_eoi(irq - PIC_IRQ_OFFSET) };
        }
        _ => (),
    }

    if called_from_ue == 1 {
        unsafe { core::ptr::write_volatile(&raw mut IN_USERSPACE, 1) };
    }
}

fn call_attached_irq(irq_id: u8, args: &InterruptInfo, called_from_ue: bool) {
    let irq_handler = IRQ_HANDLERS.lock();

    if let Some(handler) = irq_handler
        .get((irq_id) as usize)
        .and_then(|&handler| handler)
    {
        // Release the lock on the irq handler since we
        // don't know if the handler is ever going to return!
        drop(irq_handler);

        // Finally call the handler
        handler(args, called_from_ue);
    }
}

/// Set a function to be called whenever an irq is triggered.
pub fn attach_irq_handler(handler_fn: fn(&InterruptInfo, bool), irq: u8) {
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
    let mut idt = INTERRUPT_TABLE.lock();
    attach_irq!(idt, exception_handler);
    unsafe { idt.submit_table().load() };

    logln!("Attached Interrupts!");

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
        enable_interrupts();
        SHOULD_INTERRUPTS_BE_ENABLED.store(true, Ordering::Relaxed);
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
extern "C" fn syscall_handler(
    rdi: u64,
    rsi: u64,
    rdx: u64,
    _rcx: u64,
    _r8: u64,
    syscall_number: u64,
) -> u64 {
    match syscall_number {
        0 => {
            send_scheduler_event(SchedulerEvent::Exit);
            unreachable!("Should not return from exit")
        }
        1 => {
            let process = lock_ref_scheduler(|sc| sc.acquire_running_process())
                .expect("Only a running process should be able to call syscalls!");

            pub enum MemoryLocation {
                Anywhere = 0,
            }
            pub enum MemoryProtections {
                None = 0,
                ReadExecute = 1,
                ReadOnly = 2,
                ReadWrite = 3,
            }
            pub enum SysMemoryError {
                InvalidLength = 0,
                InvalidRequest = 1,
                OutOfMemory = 2,
            }

            let _location = match rdi {
                0 => MemoryLocation::Anywhere,
                _ => return SysMemoryError::InvalidRequest as u64,
            };
            let protection = match rsi {
                0 => VmPermissions::none()
                    .set_user_flag(true)
                    .set_read_flag(false)
                    .set_exec_flag(true /*FIXME: Exec flag always set*/),
                1 => VmPermissions::none()
                    .set_user_flag(true)
                    .set_read_flag(true)
                    .set_exec_flag(true),
                2 => VmPermissions::none()
                    .set_user_flag(true)
                    .set_read_flag(true)
                    .set_exec_flag(true),
                3 => VmPermissions::none()
                    .set_user_flag(true)
                    .set_read_flag(true)
                    .set_exec_flag(true)
                    .set_write_flag(true),
                _ => return SysMemoryError::InvalidRequest as u64,
            };
            let len = match rdx {
                // TODO: This is just a random constraint, we should figure out
                //       how much memory the system has.
                ..16777216 => ((rdx as usize - 1) / PAGE_4K) + 1,
                _ => return SysMemoryError::InvalidRequest as u64,
            };

            return match process.read().add_anywhere(len, protection, false) {
                Ok(region) => region.start.addr().addr() as u64,
                Err(_) => SysMemoryError::InvalidRequest as u64,
            };
        }
        69 => ::lldebug::priv_print(
            lldebug::LogKind::Log,
            "userspace",
            format_args!(
                "{}",
                core::str::from_utf8(unsafe {
                    core::slice::from_raw_parts(rdi as *const u8, rsi as usize)
                })
                .unwrap()
            ),
        ),
        _ => warnln!("Unknown syscall!"),
    }

    0
}
