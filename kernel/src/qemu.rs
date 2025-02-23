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

use arch::io::IOPort;

/// The configured debug emulator port.
///
/// The `isa-debug-exit`'s `iobase` register.
pub const QEMU_ISA_DEBUG_EXIT_IO_BASE: IOPort = IOPort::new(0xF4);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QemuExitStatus {
    Success = 0x10,
    Failure = 0x11,
}

/// Close the emulator with the given status.
///
/// # Note
/// `Success` does not close qemu with exit status '0' and instead
/// closes the emulator with '33'. The meta script knows about this
/// number and will treat it as if it did exit with status '0'.
pub fn exit_emulator(exit_status: QemuExitStatus) -> ! {
    let status = exit_status as u8;

    unsafe {
        QEMU_ISA_DEBUG_EXIT_IO_BASE.write_byte(status);

        // Busy loop if we couldn't exit
        loop {
            core::arch::asm!("hlt");
        }
    }
}
