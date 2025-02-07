/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

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

///
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
