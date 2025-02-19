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

use core::cell::SyncUnsafeCell;

use arch::{
    CpuPrivilege,
    gdt::{CodeSegmentDesc, DataSegmentDesc, GlobalDescriptorTable, TaskStateSegmentPtr},
    tss64::TaskStateSegment,
};

static KERNEL_GDT: SyncUnsafeCell<GlobalDescriptorTable<10>> =
    SyncUnsafeCell::new(GlobalDescriptorTable::new());
static KERNEL_TSS: SyncUnsafeCell<TaskStateSegment> = SyncUnsafeCell::new(TaskStateSegment::new());

pub fn init_kernel_gdt() {
    let mut gdt = GlobalDescriptorTable::new();

    gdt.store(
        1,
        CodeSegmentDesc::new64()
            .set_accessed_flag(true)
            .set_present_flag(true)
            .set_writable_flag(true),
    );

    gdt.store(
        2,
        DataSegmentDesc::new64()
            .set_accessed_flag(true)
            .set_present_flag(true)
            .set_writable_flag(true),
    );
    gdt.store(
        5,
        CodeSegmentDesc::new64()
            .set_accessed_flag(true)
            .set_present_flag(true)
            .set_writable_flag(true)
            .set_privilege_level(3),
    );
    gdt.store(
        4,
        DataSegmentDesc::new64()
            .set_accessed_flag(true)
            .set_present_flag(true)
            .set_writable_flag(true)
            .set_privilege_level(3),
    );
    gdt.store_tss(8, TaskStateSegmentPtr::new(unsafe { &*KERNEL_TSS.get() }));

    unsafe { *KERNEL_GDT.get() = gdt };
    unsafe { load_gdt() };
}

pub unsafe fn load_gdt() {
    unsafe { (&mut *KERNEL_GDT.get()).pack().load() };
}

pub fn set_stack_for_privl(rsp: *mut u8, cpu_privl: CpuPrivilege) {
    unsafe { (&mut *KERNEL_TSS.get()).set_stack_for_priv(rsp, cpu_privl) };
}

pub fn set_stack_for_ist(rsp: *mut u8, ist_id: usize) {
    unsafe { (&mut *KERNEL_TSS.get()).set_stack_for_ist(rsp, ist_id) };
}

pub unsafe fn load_tss() {
    // TODO: Make this dynamic so that the TSS can be refrenced
    unsafe { core::arch::asm!("mov ax, {tss}", "ltr ax", tss = const { (8 << 3) | 0 }) }
}
