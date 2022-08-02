/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use core::mem::size_of;
use crate::arch_x86_64::CpuPrivilegeLevel;
use crate::memory::VirtualAddress;
use crate::{serial_print, serial_println};
use crate::bitset::BitSet;
use x86_64::instructions::segmentation;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;
use x86_64::PrivilegeLevel::Ring0;


type RawHandlerFuncNe  /* No Error             */ = extern "x86-interrupt" fn(InterruptFrame);
type RawHandlerFuncE   /* With Error           */ = extern "x86-interrupt" fn(InterruptFrame, u64);
type RawHandlerFuncDne /* Diverging No Error   */ = extern "x86-interrupt" fn(InterruptFrame) -> !;
type RawHandlerFuncDe  /* Diverging With Error */ = extern "x86-interrupt" fn(InterruptFrame, u64) -> !;

/// # General Handler Function type
/// This is the function you will use when a interrupt gets called, the idt should abstract the
/// calling to each of the Raw Handlers to ensure safety. Once this type gets called, it automatically
/// will fill in the options based on what they are. So InterruptFrame, and index will always be
/// filled, but the error not not always be apparent.
pub type GeneralHandlerFunc = fn(InterruptFrame, u8, Option<u64>);

pub struct Idt([Entry; 255]);


#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptFrame {
    eip: VirtualAddress,
    code_seg: u64,
    flags: u64,
    stack_pointer: VirtualAddress,
    stack_segment: u64
}

#[cfg(not(test))]
extern "x86-interrupt" fn missing_handler(i_frame: InterruptFrame) -> ! {
    panic!("Missing Interrupt handler was called! Please add a handler to handle this interrupt! {:#x?}", i_frame);
}


impl Entry {
    pub fn new_raw_ne(gdt_select: SegmentSelector, handler: RawHandlerFuncNe) -> Self {
        let pointer = handler as u64;
        let mut blank = Self::new_blank(gdt_select);

        blank.set_handler(VirtualAddress::new(pointer));

        blank
    }

    pub fn new_raw_e(gdt_select: SegmentSelector, handler: RawHandlerFuncE) -> Self {
        let pointer = handler as u64;
        let mut blank = Self::new_blank(gdt_select);

        blank.set_handler(VirtualAddress::new(pointer));

        blank
    }

    pub fn new_raw_dne(gdt_select: SegmentSelector, handler: RawHandlerFuncDne) -> Self {
        let pointer = handler as u64;
        let mut blank = Self::new_blank(gdt_select);

        blank.set_handler(VirtualAddress::new(pointer));

        blank
    }

    pub fn new_raw_de(gdt_select: SegmentSelector, handler: RawHandlerFuncDe) -> Self {
        let pointer = handler as u64;
        let mut blank = Self::new_blank(gdt_select);

        blank.set_handler(VirtualAddress::new(pointer));

        blank
    }

    pub fn new_blank(gdt_select: SegmentSelector) -> Self {
        Entry {
            gdt_selector: gdt_select,
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }

    pub fn set_handler(&mut self, handler: VirtualAddress) {
        let pointer = handler.as_u64();

        self.pointer_low = pointer as u16;
        self.pointer_middle = (pointer >> 16) as u16;
        self.pointer_high = (pointer >> 32) as u32;
    }

    #[cfg(not(test))]
    pub fn missing() -> Self {
        Self::new_raw_dne(
            SegmentSelector::new(1, PrivilegeLevel::Ring0),
            missing_handler
        )
    }

    #[cfg(test)]
    pub fn missing() -> Self {
        Self::new_raw_ne(
            SegmentSelector::new(1, PrivilegeLevel::Ring0),
            missing_handler
        )
    }

    /// # Safety
    /// Super unsafe function as it sets all entry fields to null!
    /// This can cause undefined behavior, and maybe even crash upon loading the IDT!
    /// ---
    /// **Luckily, the IDT will not let you submit with a null entry! You must override in 2-places
    /// to override the safety of this function as its that unstable!**
    pub unsafe fn new_null() -> Self {
        Entry {
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::new_zero(),
            reserved: 0
        }
    }

    pub fn is_missing(&self) -> bool {
        let missing_ref = Self::missing();

        self.pointer_low == missing_ref.pointer_low                          &&
            self.pointer_middle == missing_ref.pointer_middle                &&
            self.pointer_high == missing_ref.pointer_high
    }

    pub fn is_null(&self) -> bool {

        self.pointer_low == 0                           &&
            self.pointer_middle == 0                    &&
            self.pointer_high == 0

    }
}

impl Idt {
    pub fn new() -> Idt {
        Idt([Entry::missing(); 255])
    }

    pub fn raw_set_handler_ne(&mut self, entry: u8, handler: RawHandlerFuncNe) {
        self.0[entry as usize] = Entry::new_raw_ne(segmentation::cs(), handler);
    }

    pub fn raw_set_handler_e(&mut self, entry: u8, handler: RawHandlerFuncE) {
        self.0[entry as usize] = Entry::new_raw_e(segmentation::cs(), handler);
    }

    pub fn raw_set_handler_dne(&mut self, entry: u8, handler: RawHandlerFuncDne) {
        self.0[entry as usize] = Entry::new_raw_dne(segmentation::cs(), handler);
    }

    pub fn raw_set_handler_de(&mut self, entry: u8, handler: RawHandlerFuncDe) {
        self.0[entry as usize] = Entry::new_raw_de(segmentation::cs(), handler);
    }

    pub fn submit_entries(&self) -> Result<IdtTablePointer, &str> {
        // This is where it gets wild, we need to make sure that the entire idt is safe and can be
        // used without the computer throwing an error, we are going to make sure that the table
        // is not malformed and can be loaded correctly.

        for i in self.0.iter() {
            // if the entry is missing, the missing type will take care of being safe
            if i.is_missing() { continue; }

            // make sure that when we load a **valid** entry, it has a GDT selector that isn't 0
            // and do this for all possible rings, we dont want to have any loop-holes
            if i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring0).0
                || i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring1).0
                || i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring2).0
                || i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring3).0
            { return Err("GDT_Selector is malformed and does not contain a valid index"); }

            // make sure the reserved bits are **always** zero
            if i.reserved != 0 {
                return Err("Reserved bits are set, this can cause a protection fault when loaded")
            }

            // make sure the must be set bits are correctly set
            if i.options.0 == 0 {
                return Err("\'Must be 1 bits\' are unset in EntryOptions");
            }

            // TODO: Add more checks to ensure each entry is set correctly
        }

        Ok(IdtTablePointer {
            base: VirtualAddress::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        })
    }

    pub unsafe fn unsafe_submit_entries(&self) -> IdtTablePointer {
        let checking_if_valid = self.submit_entries();

        if let Ok(valid) = checking_if_valid {
            // There was no errors with this idt, so loading it should be safe :)
            valid
        } else {
            serial_println!("Detected 1 or more Errors with IDT, loading this IDT can lead to undefined behavior!");

            // Do as the master said, and submit anyway!
            IdtTablePointer {
                base: VirtualAddress::new(self as *const _ as u64),
                limit: (size_of::<Self>() - 1) as u16,
            }
        }

    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed(2))]
pub struct IdtTablePointer{
    limit: u16,
    base: VirtualAddress
}

impl IdtTablePointer {
    pub fn load(&self) {
        use core::arch::asm;

        unsafe { asm!("lidt [{}]", in(reg) &self.clone(), options(readonly, nostack, preserves_flags)); };
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EntryOptions(u16);

impl EntryOptions {
    /// # Warning
    /// This has the "Must be 1-bits" **unset**! Meaning that you must set these bits before use or
    /// you risk having undefined behavior.
    pub unsafe fn new_zero() -> Self {
        EntryOptions(0)
    }

    pub fn new_minimal() -> Self {
        EntryOptions(0.set_bit_range(9..12, 0b111))
    }

    pub fn new() -> Self {
        let mut new_s = Self::new_minimal();

        // set the default options for the struct that the user might want
        new_s
            .set_cpu_prv(CpuPrivilegeLevel::Ring0)
            .enable_int(false)
            .set_present_flag(true);

        new_s
    }

    pub fn set_present_flag(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn enable_int(&mut self, enable: bool) -> &mut Self {
        self.0.set_bit(8, enable);
        self
    }

    pub fn set_cpu_prv(&mut self, cpl: CpuPrivilegeLevel) -> &mut Self {
        self.0.set_bit_range(13..15, cpl as u64);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bit_range(0..3, index as u64);
        self
    }
}


#[macro_export]
macro_rules! general_function_to_interrupt_ne {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame) {
            use core::option;

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, None);
        }

        wrapper
    }};
}

#[macro_export]
macro_rules! general_function_to_interrupt_e {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame, error_code: u64) {
            use core::option;

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, Some(error_code));
        }

        wrapper
    }};
}

#[macro_export]
macro_rules! general_function_to_interrupt_dne {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame) -> ! {
            use core::option;

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, None);

            panic!("Diverging Interrupt Function should not return!");
        }

        wrapper
    }};
}

#[macro_export]
macro_rules! general_function_to_interrupt_de {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame, error_code: u64) -> ! {
            use core::option;

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, Some(error_code));

            panic!("Diverging Interrupt Function should not return!");
        }

        wrapper
    }};
}

#[macro_export]
macro_rules! interrupt_wrapper {
    ($name: ident, 8)  => {{ $crate::general_function_to_interrupt_de!($name, 8) }};  /* Double Fault        */
    ($name: ident, 10) => {{ $crate::general_function_to_interrupt_e!($name, 10) }};  /* Invalid tss         */
    ($name: ident, 11) => {{ $crate::general_function_to_interrupt_e!($name, 11) }};  /* Segment Not Present */
    ($name: ident, 12) => {{ $crate::general_function_to_interrupt_e!($name, 12) }};  /* Stack Segment Fault */
    ($name: ident, 13) => {{ $crate::general_function_to_interrupt_e!($name, 13) }};  /* General Protection  */
    ($name: ident, 14) => {{ $crate::general_function_to_interrupt_e!($name, 14) }};  /* Page Fault          */
    ($name: ident, 17) => {{ $crate::general_function_to_interrupt_e!($name, 17) }};  /* Alignment Check     */
    ($name: ident, 18) => {{ $crate::general_function_to_interrupt_de!($name,18) }};  /* Machine Check       */
    ($name: ident, 29) => {{ $crate::general_function_to_interrupt_e!($name, 29) }};  /* VMM COMM Exception  */
    ($name: ident, 30) => {{ $crate::general_function_to_interrupt_e!($name, 30) }};  /* Security Exception  */

    ($name: ident, 9 ) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 21) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 22) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 23) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 24) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 25) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 26) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 27) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 28) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */
    ($name: ident, 31) => {{  panic!("Tried to set a reserved handler"); }};  /* RESERVED HANDLER    */

    ($name: ident, $int_n: expr) => {{ $crate::general_function_to_interrupt_ne!($name, $int_n) }};
}

#[macro_export]
macro_rules! attach_interrupt {

    ($idt: expr, $name: ident, 8 ) => /* Double Fault        */
    { $idt.raw_set_handler_de(8, $crate::interrupt_wrapper!($name, 8 )); };

    ($idt: expr, $name: ident, 10) => /* Invalid tss         */
    { $idt.raw_set_handler_e(10, $crate::interrupt_wrapper!($name, 10)); };

    ($idt: expr, $name: ident, 11) => /* Segment Not Present */
    { $idt.raw_set_handler_e(11, $crate::interrupt_wrapper!($name, 11)); };

    ($idt: expr, $name: ident, 12) => /* Stack Segment Fault */
    { $idt.raw_set_handler_e(12, $crate::interrupt_wrapper!($name, 12)); };

    ($idt: expr, $name: ident, 13) => /* General Protection  */
    { $idt.raw_set_handler_e(13, $crate::interrupt_wrapper!($name, 13)); };

    ($idt: expr, $name: ident, 14) => /* Page Fault          */
    { $idt.raw_set_handler_e(14, $crate::interrupt_wrapper!($name, 14)); };

    ($idt: expr, $name: ident, 17) => /* Alignment Check     */
    { $idt.raw_set_handler_e(17, $crate::interrupt_wrapper!($name, 17)); };

    ($idt: expr, $name: ident, 18) => /* Machine Check       */
    { $idt.raw_set_handler_de(18, $crate::interrupt_wrapper!($name, 18));};

    ($idt: expr, $name: ident, 29) => /* VMM COMM Exception  */
    { $idt.raw_set_handler_e(29, $crate::interrupt_wrapper!($name, 29)); };

    ($idt: expr, $name: ident, 30) => /* Security Exception  */
    { $idt.raw_set_handler_e(30, $crate::interrupt_wrapper!($name, 30)); };


    ($idt: expr, $name: ident, $int_n: literal) => /* Default Handler */
    { $idt.raw_set_handler_ne($int_n, $crate::interrupt_wrapper!($name, $int_n)); };

}


#[cfg(test)]
extern "x86-interrupt" fn missing_handler(i_frame: InterruptFrame) {
    serial_print!("  [MS0 CALLED]  ");
}

#[cfg(test)]
mod test_case {
    use core::arch::asm;
    use crate::arch_x86_64::idt::{EntryOptions, InterruptFrame};
    use lazy_static::lazy_static;
    use x86_64::PrivilegeLevel;
    use crate::{serial_print, serial_println};

    fn dv0_handler(i_frame: InterruptFrame, intn: u8, error: Option<u64>) {
        serial_print!("  [DV0 CALLED]  ");

        // We want this to be called and returned!

        // Do some random stuff to make sure the stack is returned correctly!
        let mut i = 0;
        for e in 33..134 {
            i += 1;
        }

        i -= 101;

        unsafe {
            let d = i == intn;

            serial_print!("[{}]  ", d);
        }
    }

    use crate::arch_x86_64::idt::Idt;

    lazy_static! {
        static ref IDT_TEST: Idt = {
            use crate::attach_interrupt;
            let mut idt = Idt::new();

            attach_interrupt!(idt, dv0_handler, 0);

            idt
        };
    }

    fn divide_by_zero_fault() {
        unsafe {
            asm!("int $0x0");
        }
    }

    fn unhandled_fault() {
        unsafe {
            asm!("int $0x01");
        }
    }

    #[test_case]
    fn test_handler_by_fault() {
        IDT_TEST.submit_entries().unwrap().load();

        divide_by_zero_fault();

        // [OK] We passed!
    }

    #[test_case]
    fn test_handler_by_not_valid() {
        IDT_TEST.submit_entries().unwrap().load();

        unhandled_fault();

        // [OK] We passed!

    }

    #[test_case]
    fn test_entry_options() {
        unsafe { assert_eq!(EntryOptions::new_zero().0, 0x00); }
        assert_eq!(EntryOptions::new_minimal().0, 0xE00);
        assert_eq!(EntryOptions::new().0, 0x8E00);
        assert_ne!(EntryOptions::new().set_present_flag(false).0, EntryOptions::new().0);
        assert_ne!(EntryOptions::new().0, EntryOptions::new_minimal().0);
    }
}

