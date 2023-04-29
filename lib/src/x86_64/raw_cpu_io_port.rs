/*!
```text
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
```

# Quantum OS Ports

according to the OSDEV Wiki --- https://wiki.osdev.org/I/O_Ports
An I/O port is usually used as a technical term for a specific address on the x86's IO bus. This
bus provides communication with devices in a fixed order and size, and was used as an alternative
to memory access. On many other architectures, there is no predefined bus for such communication
and all communication with hardware is done via memory-mapped IO. This also increasingly happens
on modern x86 hardware.


## Why would anyone need to use these?
Old cpu I/O ports are still used by some critical system components like the
PIT (Programmable Interrupt Timer) and or the PIC (Programmable Interrupt Controller) among others.

However, these functions should only be used when cpu IO is needed! CPU I/O is slow and can cause
issues on some hardware because of how legacy devices communicate with the CPU's I/O bus. These
functions are considered unsafe because of the unknown harm some I/O ports could cause on the memory
of kernel. Most times everything you do with CPU I/O ports should be in a wrapper function. That
would assure the operation is safe and checks input and outputs.


## List of IO Ports commonly used (https://wiki.osdev.org/I/O_Ports)
```text
Port range      Summary
0x0000-0x001F    The first legacy DMA controller
0x0020-0x0021    The first PIC
0x0022-0x0023    Access to the Model-Specific Registers of Cyrix processors.
0x0040-0x0047    The PIT
0x0060-0x0064    The "8042" PS/2 Controller or its predecessors, dealing with keyboards and mice.
0x0070-0x0071    The CMOS and RTC registers
0x0080-0x008F    The DMA (Page registers)
0x0092           The location of the fast A20 gate register
0x00A0-0x00A1    The second PIC
0x00C0-0x00DF    The second DMA controller, often used for SoundBlasters
0x00E9           Home of the Port E9 Hack. Used on some emulators to directly send text to the hosts' console.
0x0170-0x0177    The secondary ATA hard disk controller.
0x01F0-0x01F7    The primary ATA hard disk controller.
0x0278-0x027A    Parallel port
0x02F8-0x02FF    Second serial port
0x03B0-0x03DF    The range used for the IBM VGA, its direct predecessors, as well as any modern video card in legacy mode.
0x03F0-0x03F7    Floppy disk controller
0x03F8-0x03FF    First serial port
```
*/

use core::arch::asm;


/// # byte_in
///
/// ### Operation
/// Returns *1* Byte from the CPU I/O bus!
///
/// ### Safety
/// This function directly preforms the CPU instruction to interact with the basic CPU I/O
/// registers. Which means it has no protection or assured memory safety. This function preforms the
/// instruction `in` in x86_64 asm which directly interfaces with the CPU's I/O bus.
///
/// This function is unsafe! Please use a wrapper to assure memory safety whenever possible.
#[inline]
#[warn(unstable_features)]
pub unsafe fn byte_in(port: u16) -> u8 {
    let mut _port_value: u8 = 0;

    asm!("in al, dx", out("al") _port_value, in("dx") port, options(nomem, nostack, preserves_flags));

    _port_value
}

/// # byte_out
///
/// ### Operation
/// Puts *1* Byte onto the CPU I/O bus!
///
/// ### Safety
/// This function directly preforms the CPU instruction to interact with the basic CPU I/O
/// registers. Which means it has no protection or assured memory safety. This function preforms the
/// instruction `out` in x86_64 asm which directly interfaces with the CPU's I/O bus.
///
/// This function is unsafe! Please use a wrapper to assure memory safety whenever possible.
#[inline]
#[warn(unstable_features)]
pub unsafe fn byte_out(port: u16, data: u8) {
    asm!("out dx, al", in("dx") port, in("al") data, options(nomem, nostack, preserves_flags));
}


/// # word_in
///
/// ### Operation
/// Returns *2* Bytes from the CPU I/O bus!
///
/// ### Safety
/// This function directly preforms the CPU instruction to interact with the basic CPU I/O
/// registers. Which means it has no protection or assured memory safety. This function preforms the
/// instruction `in` in x86_64 asm which directly interfaces with the CPU's I/O bus.
///
/// This function is unsafe! Please use a wrapper to assure memory safety whenever possible.
#[inline]
#[warn(unstable_features)]
pub unsafe fn word_in(port: u16) -> u16 {
    let mut _port_value: u16 = 0;

    asm!("in ax, dx", out("ax") _port_value, in("dx") port, options(nomem, nostack, preserves_flags));

    _port_value
}


/// # word_out
///
/// ### Operation
/// Puts *2* Bytes onto the CPU I/O bus!
///
/// ### Safety
/// This function directly preforms the CPU instruction to interact with the basic CPU I/O
/// registers. Which means it has no protection or assured memory safety. This function preforms the
/// instruction `out` in x86_64 asm which directly interfaces with the CPU's I/O bus.
///
/// This function is unsafe! Please use a wrapper to assure memory safety whenever possible.
#[inline]
#[warn(unstable_features)]
pub unsafe fn word_out(port: u16, data: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") data, options(nomem, nostack, preserves_flags));
}


