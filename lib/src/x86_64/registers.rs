/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

use crate::bitset::BitSet;
use crate::x86_64::PrivlLevel;
use core::arch::asm;

/// A struct representing the 8-bit general-purpose registers in x86 architecture.
///
/// The `GPRegs8` struct contains the following 8-bit registers:
///
/// * `al`: Accumulator Low.
/// * `ah`: Accumulator High.
/// * `bl`: Base Low.
/// * `bh`: Base High.
/// * `cl`: Counter Low.
/// * `ch`: Counter High.
/// * `dl`: Data Low.
/// * `dh`: Data High.
///
/// These registers are used for a variety of purposes, including arithmetic operations,
/// addressing memory locations, and storing temporary data.
///
/// # Example
///
/// ```
/// let mut regs = GPRegs8 {
///     al: 0xAB,
///     ah: 0xCD,
///     bl: 0xEF,
///     bh: 0x12,
///     cl: 0x34,
///     ch: 0x56,
///     dl: 0x78,
///     dh: 0x9A,
/// };
///
/// assert_eq!(regs.al, 0xAB);
/// assert_eq!(regs.ah, 0xCD);
/// assert_eq!(regs.bl, 0xEF);
/// assert_eq!(regs.bh, 0x12);
/// assert_eq!(regs.cl, 0x34);
/// assert_eq!(regs.ch, 0x56);
/// assert_eq!(regs.dl, 0x78);
/// assert_eq!(regs.dh, 0x9A);
/// ```
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs8 {
    pub al: u8,
    pub ah: u8,
    pub bl: u8,
    pub bh: u8,
    pub cl: u8,
    pub ch: u8,
    pub dl: u8,
    pub dh: u8,
}

/// A struct representing the 16-bit general-purpose registers in x86 architecture.
///
/// The `GPRegs16` struct contains the following 16-bit registers:
///
/// * `ax`: Accumulator.
/// * `bx`: Base.
/// * `cx`: Counter.
/// * `dx`: Data.
/// * `bp`: Base Pointer.
/// * `sp`: Stack Pointer.
/// * `si`: Source Index.
/// * `di`: Destination Index.
///
/// These registers are used for a variety of purposes, including arithmetic operations,
/// addressing memory locations, and storing temporary data.
///
/// # Example
///
/// ```
/// let mut regs = GPRegs16 {
///     ax: 0xABCD,
///     bx: 0xEFF0,
///     cx: 0x1234,
///     dx: 0x5678,
///     bp: 0x9ABC,
///     sp: 0xDEF0,
///     si: 0x1357,
///     di: 0x2468,
/// };
///
/// assert_eq!(regs.ax, 0xABCD);
/// assert_eq!(regs.bx, 0xEFF0);
/// assert_eq!(regs.cx, 0x1234);
/// assert_eq!(regs.dx, 0x5678);
/// assert_eq!(regs.bp, 0x9ABC);
/// assert_eq!(regs.sp, 0xDEF0);
/// assert_eq!(regs.si, 0x1357);
/// assert_eq!(regs.di, 0x2468);
/// ```
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs16 {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub bp: u16,
    pub sp: u16,
    pub si: u16,
    pub di: u16,
}

/// A struct representing the 32-bit general-purpose registers in x86 architecture.
///
/// The `GPRegs32` struct contains the following 32-bit registers:
///
/// * `eax`: Accumulator.
/// * `ebx`: Base.
/// * `ecx`: Counter.
/// * `edx`: Data.
/// * `ebp`: Base Pointer.
/// * `esp`: Stack Pointer.
/// * `esi`: Source Index.
/// * `edi`: Destination Index.
///
/// These registers are used for a variety of purposes, including arithmetic operations,
/// addressing memory locations, and storing temporary data.
///
/// # Example
///
/// ```
/// let mut regs = GPRegs32 {
///     eax: 0x12345678,
///     ebx: 0x9ABCDEF0,
///     ecx: 0x13579BDF,
///     edx: 0x2468ACE0,
///     ebp: 0xAAAA5555,
///     esp: 0xBBBB6666,
///     esi: 0xCCCC7777,
///     edi: 0xDDDD8888,
/// };
///
/// assert_eq!(regs.eax, 0x12345678);
/// assert_eq!(regs.ebx, 0x9ABCDEF0);
/// assert_eq!(regs.ecx, 0x13579BDF);
/// assert_eq!(regs.edx, 0x2468ACE0);
/// assert_eq!(regs.ebp, 0xAAAA5555);
/// assert_eq!(regs.esp, 0xBBBB6666);
/// assert_eq!(regs.esi, 0xCCCC7777);
/// assert_eq!(regs.edi, 0xDDDD8888);
/// ```
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs32 {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebp: u32,
    pub esp: u32,
    pub esi: u32,
    pub edi: u32,
}

/// A struct representing the 64-bit general-purpose registers in x86 architecture.
///
/// The `GPRegs64` struct contains the following 64-bit registers:
///
/// * `rax`: Accumulator.
/// * `rbx`: Base.
/// * `rcx`: Counter.
/// * `rdx`: Data.
/// * `rsi`: Source Index.
/// * `rdi`: Destination Index.
/// * `rbp`: Base Pointer.
/// * `rsp`: Stack Pointer.
/// * `r8`: General Purpose.
/// * `r9`: General Purpose.
/// * `r10`: General Purpose.
/// * `r11`: General Purpose.
/// * `r12`: General Purpose.
/// * `r13`: General Purpose.
/// * `r14`: General Purpose.
/// * `r15`: General Purpose.
///
/// These registers are used for a variety of purposes, including arithmetic operations,
/// addressing memory locations, and storing temporary data.
///
/// # Example
///
/// ```
/// let mut regs = GPRegs64 {
///     rax: 0x1234567890ABCDEF,
///     rbx: 0x9876543210FEDCBA,
///     rcx: 0x13579BDF2468ACE0,
///     rdx: 0xE0CAB97531FD4862,
///     rsi: 0xCCCC7777AAAA5555,
///     rdi: 0xDDDD8888BBBB6666,
///     rbp: 0x1111222233334444,
///     rsp: 0x5555666677778888,
///     r8: 0xAAAA555533334444,
///     r9: 0xBBBB666611112222,
///     r10: 0xCCCC777722223333,
///     r11: 0xDDDD888833334444,
///     r12: 0x1111222255556666,
///     r13: 0x3333444466667777,
///     r14: 0x5555666688889999,
///     r15: 0x77778888AAAAFFFF,
/// };
///
/// assert_eq!(regs.rax, 0x1234567890ABCDEF);
/// assert_eq!(regs.rbx, 0x9876543210FEDCBA);
/// assert_eq!(regs.rcx, 0x13579BDF2468ACE0);
/// assert_eq!(regs.rdx, 0xE0CAB97531FD4862);
/// assert_eq!(regs.rsi, 0xCCCC7777AAAA5555);
/// assert_eq!(regs.rdi, 0xDDDD8888BBBB6666);
/// assert_eq!(regs.rbp, 0x1111222233334444);
/// assert_eq!(regs.rsp, 0x5555666677778888);
/// assert_eq!(regs.r8, 0xAAAA555533334444);
/// assert_eq!(regs.r9, 0xBBBB666611112222);
/// assert_eq!(regs.r10, 0xCCCC777722223333);
/// assert_eq!(regs.r11, 0xDDDD888833334444);
/// assert_eq!(regs.r12, 0x1111222255556666);
/// assert_eq!(regs.r13, 0x3333444466667777);
/// assert_eq!(regs.r14, 0x5555666688889999);
/// assert_eq!(regs.r15, 0x77778888AAAAFFFF);
/// ```
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct GPRegs64 {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

/// A struct representing the control registers in x86 architecture.
///
/// The `ControlRegs` struct contains the following control registers:
///
/// * `cr0`: Control Register 0.
/// * `cr2`: Control Register 2.
/// * `cr3`: Control Register 3.
/// * `cr4`: Control Register 4.
///
/// These registers control various aspects of the processor's behavior, such as
/// memory management and system protection.
///
/// # Example
///
/// ```
/// let mut regs = ControlRegs {
///     cr0: 0x12345678,
///     cr2: 0x9ABCDEF0,
///     cr3: 0x13579BDF,
///     cr4: 0x2468ACE0,
/// };
///
/// assert_eq!(regs.cr0, 0x12345678);
/// assert_eq!(regs.cr2, 0x9ABCDEF0);
/// assert_eq!(regs.cr3, 0x13579BDF);
/// assert_eq!(regs.cr4, 0x2468ACE0);
/// ```
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct ControlRegs {
    pub cr0: u32,
    pub cr2: u32,
    pub cr3: u32,
    pub cr4: u32,
}

/// A struct representing the segment registers in x86 architecture.
///
/// The `SegmentRegs` struct contains the following segment registers:
///
/// * `cs`: Code Segment.
/// * `ds`: Data Segment.
/// * `es`: Extra Segment.
/// * `ss`: Stack Segment.
/// * `fs`: F Segment.
/// * `gs`: G Segment.
///
/// These registers are used to keep track of the memory segments that are currently
/// being used by the processor.
///
/// # Example
///
/// ```
/// let mut regs = SegmentRegs {
///     cs: 0x1234,
///     ds: 0x5678,
///     es: 0x9ABC,
///     ss: 0xDEF0,
///     fs: 0x1357,
///     gs: 0x2468,
/// };
///
/// assert_eq!(regs.cs, 0x1234);
/// assert_eq!(regs.ds, 0x5678);
/// assert_eq!(regs.es, 0x9ABC);
/// assert_eq!(regs.ss, 0xDEF0);
/// assert_eq!(regs.fs, 0x1357);
/// assert_eq!(regs.gs, 0x2468);
/// ```
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
#[repr(C)]
pub struct SegmentRegs {
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub fs: u16,
    pub gs: u16,
}

/// The `CR0` control register is used in x86 architecture to control various aspects of the processor's behavior.
///
/// The `CR0` struct contains a number of methods to read and modify the various flags and settings of the `CR0` register.
///
/// # Example
///
/// ```
/// // Read the value of the CR0 register
/// let value = CR0::read_to_32();
///
/// // Check if the protected mode flag is set
/// if CR0::is_protected_mode_set() {
///     println!("Protected mode is enabled!");
/// } else {
///     println!("Protected mode is disabled.");
/// }
///
/// // Enable the write-protect flag
/// CR0::set_write_protect(true);
/// ```
pub struct CR0 {}

impl CR0 {
    /// Reads the 32-bit value of the `CR0` register and returns it as a `u32`.
    #[inline(never)]
    pub fn read_to_32() -> u32 {
        let reading_value;

        unsafe {
            asm!(
                "mov {output:e}, cr0",
                output = out(reg) reading_value
            );
        }

        reading_value
    }

    /// Writes a single bit at a given position in the `CR0` register, as specified by the `bit_pos` parameter.
    /// The bit is set to the value of the `flag` parameter, which is a boolean value indicating whether the bit should be set (true) or cleared (false).
    #[inline(never)]
    fn write_at_position(bit_pos: u32, flag: bool) {
        assert!(bit_pos <= 32);

        let read_value = Self::read_to_32();
        let set_bit_pos = 1 << bit_pos;

        let moved_bit = if flag {
            set_bit_pos
        } else {
            u32::MAX ^ set_bit_pos
        };

        let new_value = if flag {
            read_value | moved_bit
        } else {
            read_value & moved_bit
        };

        unsafe {
            asm!(
                "mov cr0, {value:e}",
                value = in(reg) new_value
            );
        }
    }

    /// Returns true if the protected mode flag is set in the `CR0` register, and false otherwise.
    pub fn is_protected_mode_set() -> bool {
        let value = Self::read_to_32();

        value & 1 == 1
    }

    /// Returns true if the monitor coprocessor flag is set in the `CR0` register, and false otherwise.
    pub fn is_monitor_coprocessor_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 1;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Returns true if the emulation flag is set in the `CR0` register, and false otherwise.
    pub fn is_enumlation_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 2;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Returns true if the task switched flag is set in the `CR0` register, and false otherwise.
    pub fn is_task_switched_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 3;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Returns true if the extension type flag is set in the `CR0` register, and false otherwise.
    pub fn is_extention_type_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 4;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Returns true if the not-write-through flag is set in the `CR0` register, and false otherwise.
    pub fn is_not_write_through_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 16;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Returns true if the alignment mask flag is set in the `CR0` register, and false otherwise.
    pub fn is_alignment_mask_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 18;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Returns true if the write-protect flag is set in the `CR0` register, and false otherwise.
    pub fn is_write_protect_set() -> bool {
        let value = Self::read_to_32();
        let bit_pos = 28;

        (value & (1 << bit_pos)) >> bit_pos == 1
    }

    /// Sets or clears the protected mode flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_protected_mode(flag: bool) {
        Self::write_at_position(0, flag);
    }

    /// Sets or clears the monitor coprocessor flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_monitor_coprocessor(flag: bool) {
        Self::write_at_position(1, flag);
    }

    /// Sets or clears the emulation flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_enumlation(flag: bool) {
        Self::write_at_position(2, flag);
    }

    /// Sets or clears the task switched flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_task_switched(flag: bool) {
        Self::write_at_position(3, flag);
    }

    /// Sets or clears the extension type flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_extention_type(flag: bool) {
        Self::write_at_position(4, flag);
    }

    /// Sets or clears the not-write-through flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_not_write_through(flag: bool) {
        Self::write_at_position(16, flag);
    }

    /// Sets or clears the alignment mask flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_alignment_mask(flag: bool) {
        Self::write_at_position(18, flag);
    }

    /// Sets or clears the write-protect flag in the `CR0` register, as specified by the `flag` parameter.
    pub fn set_write_protect(flag: bool) {
        Self::write_at_position(28, flag);
    }
}

/// The `EFLAGS` register is a special-purpose register in x86 architecture that contains various flags that control the processor's behavior and status.
///
/// The `EFLAGS` struct provides a set of methods to read and modify the various flags in the `EFLAGS` register.
///
/// # Example
///
/// ```
/// // Read the value of the EFLAGS register
/// let value = EFLAGS::read_to_u32();
///
/// // Check if the carry flag is set
/// if EFLAGS::is_carry_flag_set() {
///     println!("Carry flag is set!");
/// } else {
///     println!("Carry flag is not set.");
/// }
///
/// ```
pub struct EFLAGS {}

impl EFLAGS {
    /// Reads the value of the `EFLAGS` register and returns it as a `u32`.
    pub fn read_to_u32() -> u32 {
        let mut flags;

        unsafe {
            asm!(
            "pushf",
            "pop {flags:x}",
            flags = lateout(reg) flags,
            );
        }

        flags
    }

    /// Reads the value of a particular flag in the `EFLAGS` register, as specified by the `bit_pos` parameter.
    /// Returns `true` if the flag is set, and `false` otherwise.
    fn read_flag(bit_pos: usize) -> bool {
        let value = Self::read_to_u32();
        let flag = 1 << bit_pos;

        value & flag > 0
    }

    /// Returns `true` if the carry flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_carry_flag_set() -> bool {
        Self::read_flag(0)
    }

    /// Returns `true` if the parity flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_parity_flag_set() -> bool {
        Self::read_flag(2)
    }

    /// Returns `true` if the auxiliary flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_auxiliary_flag_set() -> bool {
        Self::read_flag(4)
    }

    /// Returns `true` if the zero flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_zero_flag_set() -> bool {
        Self::read_flag(6)
    }

    /// Returns `true` if the sign flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_sign_flag_set() -> bool {
        Self::read_flag(7)
    }

    /// Returns `true` if the trap flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_trap_flag_set() -> bool {
        Self::read_flag(8)
    }

    /// Returns `true` if the interrupt enable flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_interrupt_enable_flag_set() -> bool {
        Self::read_flag(9)
    }

    /// Returns `true` if the direction flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_direction_flag_set() -> bool {
        Self::read_flag(10)
    }

    /// Returns `true` if the overflow flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_overflow_flag_set() -> bool {
        Self::read_flag(11)
    }

    /// Returns the current CPU privilege level, as indicated by the `CPL` field of the `EFLAGS` register.
    pub fn get_cpu_priv_level() -> PrivlLevel {
        let eflags_value = Self::read_to_u32();
        let id = eflags_value.get_bits(12..14);

        PrivlLevel::new_from_usize(id as usize)
            .expect("Internel Error, 2 bit value should not be possible to outofbounds")
    }

    /// Returns `true` if the nested task flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_nested_task_flag_set() -> bool {
        Self::read_flag(14)
    }

    /// Returns `true` if the resume flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_resume_flag_set() -> bool {
        Self::read_flag(16)
    }

    /// Returns `true` if the virtual 8086 mode flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_v8086_mode_flag_set() -> bool {
        Self::read_flag(17)
    }

    /// Returns `true` if the alignment check flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_alignment_check() -> bool {
        Self::read_flag(18)
    }

    /// Returns `true` if the virtual interrupt flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_virt_interrupt_flag_set() -> bool {
        Self::read_flag(19)
    }

    /// Returns `true` if the virtual interrupt pending flag is set in the `EFLAGS` register, and `false` otherwise.
    pub fn is_virt_interrupt_pending() -> bool {
        Self::read_flag(20)
    }

    /// Returns `true` if the CPUID instruction is available on the current CPU, and `false` otherwise.
    pub fn is_cpuid_available() -> bool {
        Self::read_flag(21)
    }
}

pub struct CR1 {}

impl CR1 {
    pub unsafe fn do_you_want_to_crash_the_computer() {
        panic!("Accessing this Register will cause a fault.")
    }
}

pub struct CR2 {}

impl CR2 {
    #[cfg(target_pointer_width = "32")]
    pub fn get_32_bit_page_fault_address() -> u32 {
        let value: u32;

        unsafe {
            asm!(
            "mov eax, cr2",
            out("eax") value
            );
        }

        value
    }

    #[cfg(target_pointer_width = "64")]
    pub fn get_64_bit_page_fault_address() -> u64 {
        let value: u64;

        unsafe {
            asm!(
            "mov rax, cr2",
            out("rax") value
            );
        }

        value
    }
}

pub struct CR3 {}

/*
CR3
Bit	Label	Description	PAE	Long Mode
3	        PWT	Page-level Write-Through	        (Not used)	        (Not used if bit 17 of CR4 is 1)
4	        PCD	Page-level Cache Disable	        (Not used)	        (Not used if bit 17 of CR4 is 1)
12-31 (63)	PDBR	Page Directory Base Register	Base of PDPT	    Base of PML4T/PML5T

Bits 0-11 of the physical base address are assumed to be 0. Bits 3 and 4 of CR3 are only used when accessing a PDE in 32-bit paging without PAE.
*/

impl CR3 {
    #[cfg(target_pointer_width = "32")]
    fn read_to_u32() -> u32 {
        let value: u32;

        unsafe {
            asm!(
            "mov eax, cr3",
            out("eax") value
            );
        }

        value
    }

    #[cfg(target_pointer_width = "64")]
    fn read_to_u64() -> u64 {
        let value: u64;

        unsafe {
            asm!(
            "mov rax, cr3",
            out("rax") value
            );
        }

        value
    }

    pub fn read_to_usize() -> usize {
        #[cfg(target_pointer_width = "64")]
        return Self::read_to_u64() as usize;
        #[cfg(target_pointer_width = "32")]
        return Self::read_to_u32() as usize;
    }
}

/// Control Register 4 (CR4) is a 32-bit register in x86 and x86-64 processors that controls
/// various operating modes of the processor.
///
/// This struct provides convenient methods for reading and modifying the bits of the CR4
/// register in a safe way.
///
/// # Examples
///
/// ```no_run
/// use quantum_lib::x86_64::registers::CR4;
///
/// let is_pae_enabled = CR4::is_physical_address_extention_set();
/// println!("PAE is enabled: {}", is_pae_enabled);
///
/// ```
pub struct CR4 {}

impl CR4 {
    /// Reads the value of the CR4 register and returns it as an unsigned 32-bit integer.
    #[inline(never)]
    pub fn read_to_u32() -> u32 {
        let reading_value;

        unsafe {
            asm!(
            "mov {output:e}, cr4",
            output = out(reg) reading_value
            );
        }

        reading_value
    }

    /// Reads the value of a specific flag in the CR4 register and returns whether it is set or not.
    fn read_flag(bit_pos: usize) -> bool {
        let value = Self::read_to_u32();
        let flag = 1 << bit_pos;

        value & flag > 0
    }

    /// Returns whether the Virtual-8086 Mode Extensions feature is enabled or not.
    pub fn is_v8086_mode_extentions_set() -> bool {
        Self::read_flag(0)
    }

    /// Returns whether the Protected-Mode Virtual Interrupts feature is enabled or not.
    pub fn is_protected_mode_virtual_interrupts_set() -> bool {
        Self::read_flag(1)
    }

    /// Returns whether the Time Stamp Disable feature is enabled or not.
    pub fn is_time_stamp_disable_set() -> bool {
        Self::read_flag(2)
    }

    /// Returns whether the Debugging Extensions feature is enabled or not.
    pub fn is_debugging_extentions_set() -> bool {
        Self::read_flag(3)
    }

    /// Returns whether the Page Size Extension feature is enabled or not.
    pub fn is_page_size_extention_set() -> bool {
        Self::read_flag(4)
    }

    /// Returns whether the Physical Address Extension feature is enabled or not.
    pub fn is_physical_address_extention_set() -> bool {
        Self::read_flag(5)
    }

    /// Returns whether the Machine Check Exception feature is enabled or not.
    pub fn is_machine_check_exeption_set() -> bool {
        Self::read_flag(6)
    }

    /// Returns whether the Page Global Enable feature is enabled or not.
    pub fn is_page_global_enable_set() -> bool {
        Self::read_flag(7)
    }

    /// Returns whether the Performance Monitoring Counter Enable feature is enabled or not.
    pub fn is_performance_monitoring_counter_enable_set() -> bool {
        Self::read_flag(8)
    }

    /// Returns whether the Support for FXSAVE and FXRSTOR instructions feature is enabled or not.
    pub fn is_support_for_fxsave_and_fxrstor_instructions_set() -> bool {
        Self::read_flag(9)
    }

    /// Returns whether the Support for Unmasked SIMD Floating-Point Exceptions feature is enabled or not.
    pub fn is_support_for_unmasked_simd_fload_exceptions_set() -> bool {
        Self::read_flag(10)
    }

    /// Returns whether the User-Mode Instruction Prevention feature is enabled or not.
    pub fn is_user_mode_instruction_prevention_set() -> bool {
        Self::read_flag(11)
    }

    /// Returns whether the Virtual Machine Extensions feature is enabled or not.
    pub fn is_virtual_machine_extentions_enable_set() -> bool {
        Self::read_flag(13)
    }

    /// Returns whether the Safer Mode Extensions feature is enabled or not.
    pub fn is_safer_mode_extentions_set() -> bool {
        Self::read_flag(14)
    }

    /// Determines whether the Support for RDFSBASE/RDGSBASE/WRFSBASE/WRGSBASE instructions are supported
    pub fn is_support_for_rdfsbase_rdgsbase_wrfsbase_wrgsbase_set() -> bool {
        Self::read_flag(16)
    }

    /// Returns a boolean indicating whether the Page-Attribute Table (PAT) and
    /// the Process-Context Identifiers (PCID) feature is enabled.
    ///
    /// This function reads the Control Register 4 (CR4) to determine if the PCID
    /// feature is enabled. If bit 17 of CR4 is set, the PCID feature is enabled
    /// and this function returns `true`. If bit 17 is not set, the PCID feature
    /// is not enabled and this function returns `false`.
    ///
    /// The PCID feature is available on some Intel and AMD processors and allows
    /// for improved performance in virtualized environments by reducing the cost
    /// of address space switching.
    ///
    pub fn is_pcid_enable_set() -> bool {
        Self::read_flag(17)
    }

    /// Returns true if the `CR4` control register bit indicating support for the `XSAVE` and
    /// Processor Extended States (AVX and XSAVEOPT) instructions is set.
    ///
    /// The `XSAVE` instruction is used to save and restore processor state components in an
    /// efficient and extensible way. It supports the management of multiple extended
    /// processor states as described by the processor architecture. It enables a user to
    /// save an extended processor state component and restore it later without executing
    /// any instruction that requires that component.
    ///
    /// The `AVX` instruction set introduced a number of new registers and instructions
    /// for operations on vectors of data. It can accelerate floating point calculations,
    /// encryption, and other operations.
    ///
    /// The `XSAVEOPT` instruction was introduced to allow more efficient saving and restoring
    /// of the `XSAVE` area by eliminating unneeded state components. It is a performance
    /// optimization over `XSAVE`.
    ///
    /// The `XSAVE` and Processor Extended States features are only available on processors
    /// that support them. Attempting to execute `XSAVE` or AVX instructions on a processor
    /// that does not support these instructions can cause a general protection exception.
    ///
    pub fn is_support_for_xsafe_and_processor_extended_states_enable_set() -> bool {
        Self::read_flag(18)
    }

    /// Returns a boolean indicating whether the supervisor mode execution protection (SMEP) feature is enabled.
    pub fn is_supervisor_mode_execution_protection_enable_set() -> bool {
        Self::read_flag(20)
    }

    /// Checks whether the Supervisor Mode Access Prevention (SMAP) feature is enabled.
    ///
    /// SMAP prevents the kernel-mode code from accessing user-mode data by raising a fault
    /// whenever a privileged instruction attempts to read or write data from a user-mode
    /// address. It helps mitigate certain types of security vulnerabilities.
    ///
    /// Returns `true` if SMAP is enabled, `false` otherwise.
    pub fn is_supervisor_mode_access_prevention_enable_set() -> bool {
        Self::read_flag(21)
    }

    /// Checks whether the Protection Key (PKU) feature is enabled.
    ///
    /// PKU provides a mechanism to protect specific memory regions from accidental or
    /// malicious writes by providing a way to tag memory pages with protection keys.
    /// This feature can be used to enforce application-specific memory protection policies
    /// or to sandbox applications that are vulnerable to memory corruption attacks.
    ///
    /// Returns `true` if PKU is enabled, `false` otherwise.
    pub fn is_protection_key_enable_set() -> bool {
        Self::read_flag(22)
    }

    /// Checks whether the Protection Keys for Supervisor Mode Pages (PKE) feature is enabled.
    ///
    /// PKE provides a way to use protection keys to protect supervisor-mode pages from being
    /// accessed or modified by unprivileged code. This feature is useful in multi-tenant
    /// cloud environments where multiple virtual machines are hosted on a single physical
    /// server, and each virtual machine is assigned a set of protection keys to enforce
    /// isolation between them.
    ///
    /// Returns `true` if PKE is enabled, `false` otherwise.
    pub fn is_protection_keys_for_supervisor_mode_pages_enable_set() -> bool {
        Self::read_flag(23)
    }
}
