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

#![no_std]

pub use portal_macro::*;

/// The syscall number of the 'pass-by-enum' calling convention
pub const SYSCALL_CALLER_ID: u64 = 1;
/// Syscall Succeeded
pub const SYSCALL_OKAY_RESP: u64 = 0;
/// Syscall's inputs where not known by the kernel
pub const SYSCALL_BAD_RESP: u64 = 1;

#[cfg(feature = "client")]
pub mod syscall {
    use super::*;
    use libsys::raw_syscall;

    pub unsafe trait SyscallInput: Sized {
        /// Version ID of this syscall argument, any new release should increment the syscall number.
        fn version_id() -> u32;
    }

    pub unsafe trait SyscallOutput: Sized {
        /// Version ID of this syscall argument, any new release should increment the syscall number.
        fn version_id() -> u32;

        /// Callable function to init this data structure to an 'invalid' but _valid_-enough state
        /// for the kernel to put a response into.
        unsafe fn before_call() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    pub unsafe fn call_syscall<I: SyscallInput, O: SyscallOutput>(input: &I, output: &mut O) {
        let syscall_input_ptr = input as *const I as u64;
        let syscall_output_ptr = output as *mut O as u64;

        let syscall_packed_len = ((size_of::<I>() as u64) << 32) | (size_of::<O>() as u64);
        let syscall_packed_id = ((I::version_id() as u64) << 32) | (O::version_id() as u64);

        if syscall_packed_id == SYSCALL_OKAY_RESP || syscall_packed_id == SYSCALL_BAD_RESP {
            panic!(
                "Syscall version  cannot be a reserved request ID, please use at least '1' for both (input/output)'s version id!"
            );
        };

        match unsafe {
            raw_syscall!(
                super::SYSCALL_CALLER_ID,
                syscall_input_ptr,
                syscall_output_ptr,
                syscall_packed_len,
                syscall_packed_id
            )
        } {
            SYSCALL_OKAY_RESP => (),
            SYSCALL_BAD_RESP => panic!("QuantumOS did not understand this request!"),
            version_mismatch => {
                let kernel_version_input = version_mismatch >> 32;
                let kernel_version_output = version_mismatch & (u32::MAX as u64);

                panic!(
                    "Portal Version ID Mismatch\nKernel: {{\n  input: {},\n  output: {}\n}}\nUser: {{\n  input: {},\n  output: {}\n\n}}",
                    kernel_version_input,
                    kernel_version_output,
                    I::version_id(),
                    O::version_id()
                );
            }
        }
    }
}
