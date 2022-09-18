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

use core::arch::asm;
use core::mem::size_of;
use core::ops::{Deref, Range};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::segmentation;
use x86_64::PrivilegeLevel;
use x86_64::structures::gdt::SegmentSelector;

use crate::{serial_print, serial_println};
use crate::arch_x86_64::CpuPrivilegeLevel;
use crate::bitset::BitSet;
use crate::memory::VirtualAddress;

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

/// # IDT (interrupt descriptor table)
/// This is the root component that controls how and when ISRs will run. ISR stands for interrupt
/// service routines. These are functions that can be called when the cpu has either a fault, or
/// a software triggered interrupt. Software triggered interrupts can be used for several things
/// including 'System Calls'. System calls can be used to ask the kernel to perform privileged
/// operations like talking to I/O in a safe way, as to not allow the sand boxed application to
/// have privileges to things it shouldn't be.
///
/// # How to setup a basic IDT
/// IDT can be setup in the following way
/// ```rust
/// use quantum_os::arch_x86_64::idt::Idt;
/// use quantum_os::arch_x86_64::idt::InterruptFrame;
/// use quantum_os::{attach_interrupt, serial_println};
/// use lazy_static::lazy_static;
///
/// // This is the handler that gets called whenever `#DE` is called
/// fn divide_by_zero_handler(i_frame: InterruptFrame, int_n: u8, error_code: Option<u64>) {
///     serial_println!("EXCEPTION: DIVIDE BY ZERO {}", int_n);
/// }
///
/// // We want our IDT to be in a lazy_static to make sure it has infinite lifetime
/// lazy_static! {
///     static ref IDT: idt::Idt = {
///         let mut idt = idt::Idt::new();
///
///         // This is where we tell our IDT that we want our `divide_by_zero_handler` handler to
///         // be called whenever interrupt 0 (#DE) is called.
///         attach_interrupt!(idt, divide_by_zero_handler, 0);
///
///         idt
///     };
/// }
///
/// fn load_idt_into_memory() {
///
///     // To load our IDT we need to first `.submit_entries()`, which will turn the IDT into the
///     // more primitive structure that the CPU can understand. This structure is only generated
///     // from this command, and for safety there is no other way to make it. However, you might
///     // have noticed that there is an `.unwrap()` after `.submit_entries()`. The method that
///     // turns the IDT into the correct structure *might* fail if there is a malformed entry.
///     // So in this case we need to make sure we have a completely valid structure before we
///     // can use the `.load()`.
///     IDT.submit_entries().unwrap().load();
/// }
///
/// ```
pub struct Idt([Entry; 255]);


/// # Entry
/// This is the most basic form of the IDT. Each entry is made up of a few parts that the cpu needs
/// in a very specific order to understand. This order is as follows
/// ```text
///  pointer_low: u16
///  gdt_selector: SegmentSelector
///  options: EntryOptions
///  pointer_middle: u16
///  pointer_high: u32
///  reserved: u32
/// ```
/// The first 2-bytes of the structure are the low bytes of the pointer. This is not the entire
/// pointer, but only the first 16 bits of it. Since X86 is a weird architecture, it has odd
/// backwards compatibility, because of this we have the pointer split-up.
///
/// The `gdt_selector` is simple little structure that gives a GDT reference to every entry. This
/// mostly does not matter in 64-bit mode, but in older versions it was how privileges where controlled.
///
/// The `options` is how the cpu knows who, and when can an interrupt be called. For example, a
/// userspace application is not allowed to call `Double Fault` directly because then any program
/// can just cause the CPU to panic and the operating system would not be able to shut the program
/// down. For this reason userspace applications are only allowed to call the interrupts that the
/// kernel agrees is necessary. These are usually system calls that are allowed to be called. This
/// is because the operating system has full control on what the interrupts do and dont do.
///
/// The next two fields in our Entry struct is the second half of the pointer as mentioned before.
/// This is just the higher 48-bits of the pointer and the rest are discarded and must be 0.
///
/// `reserved` is probably the most odd thing in here. These bits **MUST** be zero. Sometimes
/// emulators will store information in them, and on real hardware it can cause a Protection Fault
/// if any of these bits are set. In special cases, these bits can also make the system super
/// unpredictable. Weird memory or untraceable errors can happen if these bits are set. For that
/// reason, when you submit the IDT it checks if these bits are zero in every entry and will Err if
/// there is even a single bit set.
///
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


/// # Interrupt Frame
/// This is probably the struct most people will need to learn. This struct is very important as it
/// represents a snapshot of the interrupt that was called. It will tell you things as simple as where
/// the cpu was executing, this can be very important for debugging. I would suggest printing out
/// the EIP on almost every fault because it will tell you exactly the instruction that caused the
/// CPU to fault. As we move on to the flags, these can be kinda specific to the interrupt. I would
/// suggest looking at the OSdev wiki for more information on the flags for your specific interrupt.
///
/// # Notes
/// This can be one of the most important things for debugging, so please learn exactly how this
/// structure can be used. It will save so much time knowing exactly what happened instead of looking
/// for what could have caused this issue in the first place.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptFrame {
    pub eip: VirtualAddress,
    pub code_seg: u64,
    pub flags: u64,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: u64,
}


/// # Fall Back Missing handler
/// This handler is attached when the IDT is created, but the missing_handler should be attached
/// once the interrupt table is loaded. This makes sure that there can be no interrupt without a
/// handler so matter where the error takes place. We want to make sure the nicer missing_handler
/// gets called, because that one is formatted currently for each and every interrupt.
extern "x86-interrupt" fn fallback_missing_handler(i_frame: InterruptFrame) -> ! {
    panic!("Fall back Missing Interrupt (UNKNOWN) handler was called!\n {:#x?}", i_frame);
}

/// # Missing handler
/// This handler is here for safety. This is called whenever an interrupt is called, but unfortunately
/// the user forgot to add a handler for that interrupt. It does not really provide much info, but
/// it causes the system to crash to make sure undefined behavior or a protection fault occurred.
#[cfg(not(test))]
fn missing_handler(i_frame: InterruptFrame, interrupt: u8, error: Option<u64>) {
    panic!("Missing Interrupt ({}) handler was called!\n {:#x?}", interrupt, i_frame);
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


    pub fn missing() -> Self {
        Self::new_raw_dne(
            SegmentSelector::new(1, PrivilegeLevel::Ring0),
            fallback_missing_handler,
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
            reserved: 0,
        }
    }

    pub fn is_fallback_missing(&self) -> bool {
        let missing_ref = Self::missing();

        self.pointer_low == missing_ref.pointer_low &&
            self.pointer_middle == missing_ref.pointer_middle &&
            self.pointer_high == missing_ref.pointer_high
    }

    pub fn is_null(&self) -> bool {
        self.pointer_low == 0 &&
            self.pointer_middle == 0 &&
            self.pointer_high == 0
    }
}

impl Idt {
    pub fn new() -> Idt {
        use crate::attach_interrupt;

        // Attach the nicer missing_handler to the idt this
        // one will be correctly formatted for each interrupt!
        let mut idt = Idt([Entry::missing(); 255]);
        attach_interrupt!(idt, missing_handler, 0..255);

        idt
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
            if i.is_fallback_missing() { continue; }

            // make sure that when we load a **valid** entry, it has a GDT selector that isn't 0
            // and do this for all possible rings, we dont want to have any loop-holes
            if i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring0).0
                || i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring1).0
                || i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring2).0
                || i.gdt_selector.0 == SegmentSelector::new(0, PrivilegeLevel::Ring3).0
            { return Err("GDT_Selector is malformed and does not contain a valid index"); }

            // make sure the reserved bits are **always** zero
            if i.reserved != 0 {
                return Err("Reserved bits are set, this can cause a protection fault when loaded");
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

    pub fn check_for_entry(&self, interrupt: u8) -> bool {
        let entry = &self.0[interrupt as usize];

        !(entry.is_null() || entry.is_fallback_missing())
    }

    pub fn remove_entry(&mut self, interrupt: u8) {
        if self.check_for_entry(interrupt) {
            self.0[interrupt as usize] = Entry::missing();

            crate::attach_interrupt!(self, missing_handler, interrupt..(interrupt+1));
        }
    }
}

/// # IDT Table Pointer
/// This is the pointer to where you stored your IDT, but in a structure the CPU will accept. This
/// structure is created when you submit_entries in the IDT. This is the only way of making this
/// because the IDT controls such fundamental CPU operations, it would be very unsafe for anyone
/// to make this structure. If this is malformed, it can cause the entire operating system to have
/// issues that are almost untraceable. Whenever the CPU tries to call an interrupt and this does
/// not point a valid IDT, it will call a `General Protection Fault`. This fault then isn't handled
/// (because again the idt doesn't exist) so a `Double Fault` will be called. Once the `Double Fault`
/// is called, the cpu already knows it can't recover. This isn't the end for our poor cpu, however,
/// this interrupt isn't handled again, so it causes a `Triple Fault`. This is finally the end for
/// the cpu. This interrupt can not be handled and the cpu is forced to reset the system. Once the
/// system is reset, however, the entire process will happen over and over. This is generally considered
/// "boot looping", because the system will boot and then shutdown over and over.
///
/// This is why it is very important that this is made correctly, and for that reason is can only be
/// made through the IDT.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(2))]
pub struct IdtTablePointer {
    limit: u16,
    base: VirtualAddress,
}

lazy_static! {
    pub static ref IDT_TABLE_POINTER: Mutex<IdtTablePointer> = {
        Mutex::new(Idt::new().submit_entries().expect("Could not Init temp IDT!"))
    };
}



impl IdtTablePointer {
    pub fn load(&self) {
        IDT_TABLE_POINTER.lock().copy_from(*self);

        unsafe { asm!("lidt [{}]", in(reg) IDT_TABLE_POINTER.lock().deref(), options(readonly, nostack, preserves_flags)); };
    }

    pub fn copy_from(&mut self, other: Self) {
        self.limit = other.limit;
        self.base = other.base;
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
        EntryOptions(0.set_bits(9..12, 0b111))
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
        self.0.set_bits(13..15, cpl as u64);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index as u64);
        self
    }
}

/// # Extra Handler Info
/// If you ask about the interrupt, this struct will be created and populated with the
/// extra information that the isr might benefit from knowing. This struct will tell you
/// basic info regarding if the interrupt is allowed to terminate, or should return.
///
/// # Future
/// This struct should be populated with scheduler information regarding the process that the
/// interrupt might need to be handled.
///
/// # Interrupts
/// ```text
/// | Interrupt |    Type   |  Function Parameters |
/// |-----------|-----------|----------------------|
/// |   0 - 7   |   NO ERR  |   None          ()   |
/// |     8     |  DV W ERR |   Some         -> !  |
/// |     9     |  Reserved |         PANIC        |
/// |  10 - 14  |   W ERR   |   Some          ()   |
/// |  15 - 16  |   NO ERR  |   None          ()   |
/// |    17     |   W ERR   |   Some          ()   |
/// |    18     |  DV W ERR |   Some         -> !  |
/// |  19 - 20  |   NO ERR  |   None          ()   |
/// |  21 - 28  |  Reserved |         PANIC        |
/// |  29 - 30  |   W ERR   |   Some          ()   |
/// |    31     |  Reserved |         PANIC        |
/// |-----------|-----------|----------------------|
/// ```
///
/// ## Interrupts 0-31
/// These are system generated interrupts. These interrupts are usually called 'Faults'. Most
/// Interrupts in this range can be handled and returned, but there are some that can't. Commonly
/// though, the interrupts are usually caused by a fault of some kind. This could be as simple as a
/// Divide-By-Zero.
///
/// ```text
/// | Number |      Interrupt Name      | Short Name |
/// |--------|--------------------------|------------|
/// |    0   |      Divide by Zero      |    #DE     |
/// |    1   |          Debug           |    #DB     |
/// |    2   |  NON Maskable Interrupt  |    NMI     |
/// |    3   |       BreakPoint         |    #BP     |
/// |    4   |        OverFlow          |    #OF     |
/// |    5   |   Bound Range Exceeded   |    #BR     |
/// |    6   |      Invalid Opcode      |    #UD     |
/// |    7   |   Device not Available   |    #NM     |
/// |    8   |       Double Fault       |    #DF     |
/// |    9   |         RESERVED         |    RSV     |
/// |   10   |        Invalid TSS       |    #TS     |
/// |   11   |    Segment not Present   |    #NP     |
/// |   12   |    Stack Segment Fault   |    #SS     |
/// |   13   | General Protection Fault |    #GP     |
/// |   14   |        Page Fault        |    #PF     |
/// |   15   |         RESERVED         |    RSV     |
/// |   16   |    X87 Floating Point    |    #MF     |
/// |   17   |     Alignment Check      |    #AC     |
/// |   18   |      Machine Check       |    #MC     |
/// |   19   |    SIMD Floating Point   |    #XF     |
/// |   20   |      Virtualization      |     V      |
/// |   21   |         RESERVED         |    RSV     |
/// |   22   |         RESERVED         |    RSV     |
/// |   23   |         RESERVED         |    RSV     |
/// |   24   |         RESERVED         |    RSV     |
/// |   25   |         RESERVED         |    RSV     |
/// |   26   |         RESERVED         |    RSV     |
/// |   27   |         RESERVED         |    RSV     |
/// |   28   |         RESERVED         |    RSV     |
/// |   29   |    VMM Comm Exception    |    #VC     |
/// |   30   |    Security Exception    |    #SX     |
/// |   31   |         RESERVED         |    RSV     |
/// |--------|--------------------------|------------|
/// ```
pub struct ExtraHandlerInfo {
    /// *This type of interrupt should not return and will cause a panic if returned*
    pub should_handler_diverge: bool,

    /// *Setting a reserved interrupt is not recommended, and you should return or panic from the
    /// interrupt if this flag is true!*
    pub reserved_interrupt: bool,

    /// should interrupt be quite
    pub quiet_interrupt: bool,

    /// Interrupt name
    pub interrupt_name: &'static str,
}

impl ExtraHandlerInfo {
    pub fn new(interrupt_id: u8) -> ExtraHandlerInfo {
        let info = ExtraHandlerInfo {
            should_handler_diverge: false,
            reserved_interrupt: false,
            quiet_interrupt: QUIET_INTERRUPT_VECTOR.lock()[interrupt_id as usize],
            interrupt_name: "unnamed interrupt",
        };

        match interrupt_id {
             0 => { ExtraHandlerInfo { interrupt_name: "Divide by Zero", ..info } },
             1 => { ExtraHandlerInfo { interrupt_name: "Debug", ..info } },
             2 => { ExtraHandlerInfo { interrupt_name: "NON Maskable Interrupt", ..info } },
             3 => { ExtraHandlerInfo { interrupt_name: "BreakPoint", ..info } },
             4 => { ExtraHandlerInfo { interrupt_name: "OverFlow", ..info } },
             5 => { ExtraHandlerInfo { interrupt_name: "Bound Range Exceeded", ..info } },
             6 => { ExtraHandlerInfo { interrupt_name: "Invalid Opcode", ..info } },
             7 => { ExtraHandlerInfo { interrupt_name: "Device not Available", ..info } },
             8 => { ExtraHandlerInfo { should_handler_diverge: true, interrupt_name: "Double Fault", ..info } },
            10 => { ExtraHandlerInfo { interrupt_name: "Invalid TSS", ..info } },
            11 => { ExtraHandlerInfo { interrupt_name: "Segment not Present", ..info } },
            12 => { ExtraHandlerInfo { interrupt_name: "Stack Segment Fault", ..info } },
            13 => { ExtraHandlerInfo { interrupt_name: "General Protection Fault", ..info } },
            14 => { ExtraHandlerInfo { interrupt_name: "Page Fault", ..info } },
            16 => { ExtraHandlerInfo { interrupt_name: "X87 Floating Point", ..info } },
            17 => { ExtraHandlerInfo { interrupt_name: "Alignment Check", ..info } },
            18 => { ExtraHandlerInfo { should_handler_diverge: true, interrupt_name: "Machine Check", ..info } },
            19 => { ExtraHandlerInfo { interrupt_name: "SIMD Floating Point", ..info } },
            20 => { ExtraHandlerInfo { interrupt_name: "Virtualization", ..info } },
            29 => { ExtraHandlerInfo { interrupt_name: "VMM Comm Exception", ..info } },
            30 => { ExtraHandlerInfo { interrupt_name: "Security Exception", ..info } },

            9 | 21..=28 | 31 | 15 => { ExtraHandlerInfo { reserved_interrupt: true, ..info } },

            _ => {
                info
            }
        }
    }
}

lazy_static! {
    static ref QUIET_INTERRUPT_VECTOR: Mutex<[bool; 255]> = {
        Mutex::new([false; 255])
    };
}

/// # Set Quite Interrupt
/// This function will set a flag on the ExtraHandlerInfo struct to let your handler know it should
/// not produce any output for that interrupt.
///
/// # Usage
/// ```rust
/// use quantum_os::arch_x86_64::idt::set_quiet_interrupt;
///
/// set_quiet_interrupt(10, true); // Sets "Invalid TSS" to not produce output when called
/// set_quiet_interrupt(12, false); // Sets "Stack Segment Fault" to produce output when called
///
/// // Default is NON-QUITE output!
///
/// ```
#[inline]
pub fn set_quiet_interrupt(interrupt_id: u8, quiet: bool) {
    QUIET_INTERRUPT_VECTOR.lock()[interrupt_id as usize] = quiet;
}

/// # Set Quite Interrupt
/// This function will set a flag on the ExtraHandlerInfo struct to let your handler know it should
/// not produce any output for that interrupt.
///
/// # Usage
/// ```rust
/// use quantum_os::arch_x86_64::idt::set_quiet_interrupt_range;
///
/// set_quiet_interrupt_range(10..15, true); // Sets "Invalid TSS" to not produce output when called
/// set_quiet_interrupt_range(12..255, false); // Sets "Stack Segment Fault" to produce output when called
///
/// // Default is NON-QUITE output!
///
/// ```
#[inline]
pub fn set_quiet_interrupt_range(interrupt_id: Range<u8>, quiet: bool) {
    for i in interrupt_id {
        set_quiet_interrupt(i, quiet);
    }
}


/// # Interrupt Tester
/// This will simply just call a basic interrupt to test your IDT. This interrupt will call
/// interrupt 1 - or simply known as 'Debug'.
#[inline]
pub fn interrupt_tester() {
    unsafe {
        asm!("int $0x01");
    }
}


/// # General Function To Interrupt (No Error)
/// This is a general wrapper around a `GeneralHandlerFunc` type and the corresponding interrupt
/// type. This generates a new function that matches what the cpu will be expecting, then will pass
/// the arguments it gathers to the called function. This allows one simple general type of function
/// to handle many different types of interrupts regardless of how each interrupt needs to be
/// structured. Some interrupts have error codes pushed to the stack, and others don't. This is why
/// we need to wrap your function around a "wrapper" to make sure you are not trying to access unsafe
/// memory. This also makes sure if your function is diverging, it will panic before the function
/// returns to stop a triple fault from happening.
#[macro_export]
macro_rules! general_function_to_interrupt_ne {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame) {

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, None);
        }

        wrapper
    }};
}

/// # General Function To Interrupt (With Error)
/// This is a general wrapper around a `GeneralHandlerFunc` type and the corresponding interrupt
/// type. This generates a new function that matches what the cpu will be expecting, then will pass
/// the arguments it gathers to the called function. This allows one simple general type of function
/// to handle many different types of interrupts regardless of how each interrupt needs to be
/// structured. Some interrupts have error codes pushed to the stack, and others don't. This is why
/// we need to wrap your function around a "wrapper" to make sure you are not trying to access unsafe
/// memory. This also makes sure if your function is diverging, it will panic before the function
/// returns to stop a triple fault from happening.
#[macro_export]
macro_rules! general_function_to_interrupt_e {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame, error_code: u64) {

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, Some(error_code));
        }

        wrapper
    }};
}

/// # General Function To Interrupt (Diverging No Error)
/// This is a general wrapper around a `GeneralHandlerFunc` type and the corresponding interrupt
/// type. This generates a new function that matches what the cpu will be expecting, then will pass
/// the arguments it gathers to the called function. This allows one simple general type of function
/// to handle many different types of interrupts regardless of how each interrupt needs to be
/// structured. Some interrupts have error codes pushed to the stack, and others don't. This is why
/// we need to wrap your function around a "wrapper" to make sure you are not trying to access unsafe
/// memory. This also makes sure if your function is diverging, it will panic before the function
/// returns to stop a triple fault from happening.
#[macro_export]
macro_rules! general_function_to_interrupt_dne {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame) -> ! {

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, None);

            panic!("Diverging Interrupt Function should not return!");
        }

        wrapper
    }};
}

/// # General Function To Interrupt (Diverging With Error)
/// This is a general wrapper around a `GeneralHandlerFunc` type and the corresponding interrupt
/// type. This generates a new function that matches what the cpu will be expecting, then will pass
/// the arguments it gathers to the called function. This allows one simple general type of function
/// to handle many different types of interrupts regardless of how each interrupt needs to be
/// structured. Some interrupts have error codes pushed to the stack, and others don't. This is why
/// we need to wrap your function around a "wrapper" to make sure you are not trying to access unsafe
/// memory. This also makes sure if your function is diverging, it will panic before the function
/// returns to stop a triple fault from happening.
#[macro_export]
macro_rules! general_function_to_interrupt_de {
    ($name: ident, $int_num: expr) => {{
        extern "x86-interrupt" fn wrapper(i_frame: InterruptFrame, error_code: u64) -> ! {

            let function = $name as $crate::arch_x86_64::idt::GeneralHandlerFunc;

            function(i_frame, $int_num, Some(error_code));

            panic!("Diverging Interrupt Function should not return!");
        }

        wrapper
    }};
}

/// # Interrupt Match Wrapper
/// This wraps the match statement that filters out the type of interrupt. This is so it can be used
/// in other macros to provide easier to read code!
#[macro_export]
macro_rules! interrupt_match_wrapper {
    ($idt: expr, $name: ident, $int_n: tt) => {
        match $int_n as usize {
            8 | 18 => {
                $idt.raw_set_handler_de($int_n, $crate::general_function_to_interrupt_de!($name, $int_n));
            },
            10..=14 | 17 | 29 | 30 => {
                 $idt.raw_set_handler_e($int_n, $crate::general_function_to_interrupt_e!($name, $int_n));
            },
            9 | 21..=28 | 31 => { panic!("Tried to set a reserved handler"); }
            _ => {
                $idt.raw_set_handler_ne($int_n, $crate::general_function_to_interrupt_ne!($name, $int_n));
            }
        }
    };
}

/// # Interrupt Match Wrapper Recursive
/// This macro gets around the rust problem of passing a non-const value into a function. This is fixed
/// by spawning this macro 255 times, and detecting if that macro contains an index that is inside the
/// range provided. This is overly complicated, but its a rust limitation on how macros work.
#[macro_export]
macro_rules! interrupt_match_wrapper_recursive {
    ($idt: expr, $name: ident, $range: expr, $bit7:tt, $bit6:tt, $bit5:tt, $bit4:tt, $bit3:tt, $bit2:tt, $bit1:tt, $bit0:tt) => {{

        // Now that we have all 8 bits populated, we have to detect if this permutation is inside
        // the range that we provided. If this permutation is with-in the range, then it gets to live
        // otherwise it dies. This leaves us with just a macro that fits our range. Then we can finally
        // call interrupt_match_wrapper! to have that permutation added.
        const INDEX: u8 = $bit0 | ($bit1 << 1) | ($bit2 << 2) | ($bit3 << 3) | ($bit4 << 4) | ($bit5 << 5) | ($bit6 << 6) | ($bit7 << 7);

        // This is to make sure we dont hit any reserved handlers
        if (INDEX != 9 && INDEX != 31 && !((21..=28).contains(&INDEX))) {

            // Check if the index is inside the range
            if $range.contains(&INDEX) { $crate::interrupt_match_wrapper!($idt, $name, INDEX); }
        }
    }};

    ($idt: expr, $name: ident, $range: expr $(, $bits:tt)*) => {

        // Spawn every permutation of bits 00000000 -> 11111111
        $crate::interrupt_match_wrapper_recursive!($idt, $name, $range $(, $bits)*, 0);
        $crate::interrupt_match_wrapper_recursive!($idt, $name, $range $(, $bits)*, 1);
    };
}



/// # Attach Interrupt
/// This macro will attach a `GeneralHandlerFunc` type of function to the IDT. It will automatically
/// pick-out the type of interrupt and wrap it accordingly.
///
/// ## Interrupt Types
/// The following interrupts will produce the following behavior:
/// ```text
/// | Interrupt |    Type   |  Function Parameters |
/// |-----------|-----------|----------------------|
/// |   0 - 7   |   NO ERR  |   None          ()   |
/// |     8     |  DV W ERR |   Some         -> !  |
/// |     9     |  Reserved |         PANIC        |
/// |  10 - 14  |   W ERR   |   Some          ()   |
/// |  15 - 16  |   NO ERR  |   None          ()   |
/// |    17     |   W ERR   |   Some          ()   |
/// |    18     |  DV W ERR |   Some         -> !  |
/// |  19 - 20  |   NO ERR  |   None          ()   |
/// |  21 - 28  |  Reserved |         PANIC        |
/// |  29 - 30  |   W ERR   |   Some          ()   |
/// |    31     |  Reserved |         PANIC        |
/// |-----------|-----------|----------------------|
/// ```
///
/// ## Interrupts 0-31
/// These are system generated interrupts. These interrupts are usually called 'Faults'. Most
/// Interrupts in this range can be handled and returned, but there are some that can't. Commonly
/// though, the interrupts are usually caused by a fault of some kind. This could be as simple as a
/// Divide-By-Zero.
///
/// ```text
/// | Number |      Interrupt Name      | Short Name |
/// |--------|--------------------------|------------|
/// |    0   |      Divide by Zero      |    #DE     |
/// |    1   |          Debug           |    #DB     |
/// |    2   |  NON Maskable Interrupt  |    NMI     |
/// |    3   |       BreakPoint         |    #BP     |
/// |    4   |        OverFlow          |    #OF     |
/// |    5   |   Bound Range Exceeded   |    #BR     |
/// |    6   |      Invalid Opcode      |    #UD     |
/// |    7   |   Device not Available   |    #NM     |
/// |    8   |       Double Fault       |    #DF     |
/// |    9   |         RESERVED         |    RSV     |
/// |   10   |        Invalid TSS       |    #TS     |
/// |   11   |    Segment not Present   |    #NP     |
/// |   12   |    Stack Segment Fault   |    #SS     |
/// |   13   | General Protection Fault |    #GP     |
/// |   14   |        Page Fault        |    #PF     |
/// |   15   |         RESERVED         |    RSV     |
/// |   16   |    X87 Floating Point    |    #MF     |
/// |   17   |     Alignment Check      |    #AC     |
/// |   18   |      Machine Check       |    #MC     |
/// |   19   |    SIMD Floating Point   |    #XF     |
/// |   20   |      Virtualization      |     V      |
/// |   21   |         RESERVED         |    RSV     |
/// |   22   |         RESERVED         |    RSV     |
/// |   23   |         RESERVED         |    RSV     |
/// |   24   |         RESERVED         |    RSV     |
/// |   25   |         RESERVED         |    RSV     |
/// |   26   |         RESERVED         |    RSV     |
/// |   27   |         RESERVED         |    RSV     |
/// |   28   |         RESERVED         |    RSV     |
/// |   29   |    VMM Comm Exception    |    #VC     |
/// |   30   |    Security Exception    |    #SX     |
/// |   31   |         RESERVED         |    RSV     |
/// |--------|--------------------------|------------|
/// ```
///
///
#[macro_export]
macro_rules! attach_interrupt {
    ($idt: expr, $name: ident, $int_n: literal) => {
        $crate::interrupt_match_wrapper!($idt, $name, $int_n);
    };

    ($idt: expr, $name: ident, $int_n: expr) => {
        $crate::interrupt_match_wrapper_recursive!($idt, $name, $int_n);
    };
}


/// # Remove Interrupt
/// This macro will remove the current interrupt handler that was set.
///
/// ## Basic Use:
/// ```rust
/// use quantum_os::remove_interrupt;
///
///
/// remove_interrupt!(your_idt, 0); // removes #DE
/// remove_interrupt!(your_idt, 1..10); // removes a range of interrupts
///
/// ```
#[macro_export]
macro_rules! remove_interrupt {
    ($idt: expr, $int_n: literal) => {
        $idt.remove_entry($int_n);
    };

    ($idt: expr, $int_n: expr) => {
        for i in $int_n {
            $idt.remove_entry(i);
        }
    };
}

// --- Tests ---
// These are to ensure the system is working as intended.

#[cfg(test)]
fn missing_handler(_i_frame: InterruptFrame, _interrupt: u8, _error: Option<u64>) { }

#[cfg(test)]
mod test_case {
    use core::arch::asm;

    use lazy_static::lazy_static;
    use spin::Mutex;
    use x86_64::PrivilegeLevel;

    use crate::{serial_print, serial_println};
    use crate::arch_x86_64::idt::{EntryOptions, InterruptFrame};
    use crate::arch_x86_64::idt::Idt;

    fn dv0_handler(i_frame: InterruptFrame, intn: u8, error: Option<u64>) {
        // We want this to be called and returned!

        // Do some random stuff to make sure the stack is returned correctly!
        let mut i = 0;
        for _ in 33..134 {
            i += 1;
        }

        i -= 101;

        let d = i == intn;

    }

    lazy_static! {
        static ref IDT_TEST: Mutex<Idt> = {
            use crate::attach_interrupt;
            let mut idt = Idt::new();

            attach_interrupt!(idt, dv0_handler, 0);

            Mutex::new(idt)
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
        {
            let m_idt = IDT_TEST.lock();

            m_idt.submit_entries().unwrap().load();
        }

        divide_by_zero_fault();

        // [OK] We passed!
    }

    #[test_case]
    fn test_handler_by_not_valid() {


        let mut m_idt = IDT_TEST.lock();

        m_idt.submit_entries().unwrap().load();

        unhandled_fault();

        remove_interrupt!(m_idt, 0..255);

        m_idt.submit_entries().unwrap().load();

        divide_by_zero_fault();

        // [OK] We passed!
    }

    #[test_case]
    fn test_add_and_remove_case() {
        let mut m_idt = IDT_TEST.lock();

        attach_interrupt!(m_idt, dv0_handler, 0..255);

        m_idt.submit_entries().unwrap().load();

        remove_interrupt!(m_idt, 0..255);

        m_idt.submit_entries().unwrap().load();


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

