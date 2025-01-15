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

use arch::gdt::{CodeSegmentDesc, DataSegmentDesc, GlobalDescriptorTable, TaskStateSegmentPtr};
use spin::RwLock;

static KERNEL_GDT: RwLock<GlobalDescriptorTable<7>> = RwLock::new(GlobalDescriptorTable::new());

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
        3,
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
    // gdt.store_tss(5, TaskStateSegmentPtr::new(todo!()));

    *KERNEL_GDT.write() = gdt;
    unsafe { load_gdt() };
}

pub unsafe fn load_gdt() {
    unsafe { (&mut *KERNEL_GDT.as_mut_ptr()).pack().load() };
}
