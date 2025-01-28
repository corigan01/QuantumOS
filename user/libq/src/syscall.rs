/*
  ____                 __               __  __
 / __ \__ _____ ____  / /___ ____ _    / / / /__ ___ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /_/ (_-</ -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/  \____/___/\__/_/
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

/// Preform a raw system call to QuantumOS.
///
/// This is a wrapper around an assembly block that preforms a system call.
#[macro_export]
macro_rules! raw_syscall {
    (__priv_custom_asm, $($tt:tt)*) => {
        ::core::arch::asm!(
            // "int 0x80",
            "syscall",
            $($tt)*
            clobber_abi("system")
        );
    };
    ($syscall_number:expr) => {{
        let mut syscall_return: u64 = { $syscall_number } as u64;

        $crate::raw_syscall!(__priv_custom_asm,
            inlateout("rax") syscall_return,
            out("rcx") _,
            out("r11") _,
        );

        syscall_return
    }};
    ($syscall_number:expr, $arg1:expr) => {{
        let mut syscall_return: u64 = { $syscall_number } as u64;

        $crate::raw_syscall!(__priv_custom_asm,
            inlateout("rax") syscall_return,
            in("rdi") $arg1,
            out("rcx") _,
            out("r11") _,
        );

        syscall_return
    }};
    ($syscall_number:expr, $arg1:expr, $arg2:expr) => {{
        let mut syscall_return: u64 = { $syscall_number } as u64;

        $crate::raw_syscall!(__priv_custom_asm,
            inlateout("rax") syscall_return,
            in("rdi") $arg1,
            in("rsi") $arg2,
            out("rcx") _,
            out("r11") _,
        );

        syscall_return
    }};
    ($syscall_number:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {{
        let mut syscall_return: u64 = { $syscall_number } as u64;

        $crate::raw_syscall!(__priv_custom_asm,
            inlateout("rax") syscall_return,
            in("rdi") $arg1,
            in("rsi") $arg2,
            in("rdx") $arg3,
            out("rcx") _,
            out("r11") _,
        );

        syscall_return
    }};
    ($syscall_number:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {{
        let mut syscall_return: u64 = { $syscall_number } as u64;

        $crate::raw_syscall!(__priv_custom_asm,
            inlateout("rax") syscall_return,
            in("rdi") $arg1,
            in("rsi") $arg2,
            in("rdx") $arg3,
            inout("rcx") $arg4 => _,
            out("r11") _,
        );

        syscall_return
    }};
    ($syscall_number:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {{
        let mut syscall_return: u64 = { $syscall_number } as u64;

        $crate::raw_syscall!(__priv_custom_asm,
            inlateout("rax") syscall_return,
            in("rdi") $arg1,
            in("rsi") $arg2,
            in("rdx") $arg3,
            inout("rcx") $arg4 => _,
            in("r8") $arg5,
            out("r11") _,
        );

        syscall_return

    }};
}

/// QuantumOS Exit SyscallID
pub const QOS_SYSCALL_NUMBER_EXIT: usize = 0;
/// QuantumOS Memory SyscallID
pub const QOS_SYSCALL_NUMBER_MEMORY: usize = 1;
/// QuantumOS Debug Output SyscallID
pub const QOS_SYSCALL_NUMBER_DEBUG: usize = 69;

/// Possiable errors for `debug` system call.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub enum SysDebugResp {
    Okay,
    PtrInvalid,
    LenInvalid,
}

// TODO: This should be put into a macro because all syscalls will have this same structure.
impl SysDebugResp {
    pub fn unwrap(self) {
        match self {
            SysDebugResp::Okay => (),
            SysDebugResp::PtrInvalid => panic!("SysDebugResp: unwrap - Input Ptr was invalid!"),
            SysDebugResp::LenInvalid => panic!("SysDebugResp: unwrap - Input Length was invalid!"),
        }
    }
}

/// Write to the kernel's debug output
pub unsafe fn debug_syscall(msg: &str) -> SysDebugResp {
    let syscall_resp =
        unsafe { raw_syscall!(QOS_SYSCALL_NUMBER_DEBUG, msg.as_ptr(), msg.bytes().len()) };

    match syscall_resp {
        0 => SysDebugResp::Okay,
        1 => SysDebugResp::PtrInvalid,
        2 => SysDebugResp::LenInvalid,
        // _ => unreachable!("Kernel should only repond with SysDebugResp, kernel error?"),
        _ => SysDebugResp::Okay,
    }
}

/// Possible Exit Reasons
#[derive(Debug, Clone, Copy)]
pub enum SysExitCode {
    Success,
    Failure,
    Other(u8),
}

/// Exit the program
///
/// # Note
/// This syscall *cannot* fail, and will be guaranteed to exit the program!
pub unsafe fn exit_syscall(exit_reason: SysExitCode) -> ! {
    let sys_exit_code: u64 = match exit_reason {
        SysExitCode::Success => 0,
        SysExitCode::Failure => 1,
        SysExitCode::Other(e) => e as u64,
    };

    unsafe { raw_syscall!(QOS_SYSCALL_NUMBER_EXIT, sys_exit_code) };
    unreachable!("Exit syscall should never return!")
}

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryLocation {
    Anywhere = 0,
}

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryProtections {
    None = 0,
    ReadExecute = 1,
    ReadOnly = 2,
    ReadWrite = 3,
}

/// Possiable errors for `memory` system call.
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum SysMemoryError {
    InvalidLength = 0,
    InvalidRequest = 1,
    OutOfMemory = 2,
}

pub unsafe fn map_memory(
    loc: MemoryLocation,
    prot: MemoryProtections,
    length: usize,
) -> Result<*mut u8, SysMemoryError> {
    let arg0 = loc as u64;
    let arg1 = prot as u64;
    let arg2 = length as u64;

    let resp = unsafe { raw_syscall!(QOS_SYSCALL_NUMBER_MEMORY, arg0, arg1, arg2) };

    match resp {
        // Since these are kernel PTRs, they should never be valid in userspace!
        0 => Err(SysMemoryError::InvalidLength),
        1 => Err(SysMemoryError::InvalidRequest),
        2 => Err(SysMemoryError::OutOfMemory),
        // Our stack ptr is the highest possible ptr in userspace
        e if e < 0x7fffffffffff => Ok(e as *mut u8),
        _ => unreachable!("Kernel should only repond with SysMemoryError, kernel error?"),
    }
}
