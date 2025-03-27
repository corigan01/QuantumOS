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

#![no_std]

pub mod alloc;
pub mod debug;
pub mod sync;

// Import syscall interface
pub use quantum_portal::sys_client::*;
pub use quantum_portal::*;

/// Termination trait for `main` to convert `()` or `Result<O, E>` to an exit status.
pub trait QuantumTermination {
    fn exit_status(self) -> ExitReason;
}

impl QuantumTermination for () {
    fn exit_status(self) -> ExitReason {
        ExitReason::Success
    }
}

impl<E: core::error::Error> QuantumTermination for Result<(), E> {
    fn exit_status(self) -> ExitReason {
        match self {
            Ok(_) => ExitReason::Success,
            Err(err) => {
                dbugln!("Failure {err}");
                ExitReason::Failure
            }
        }
    }
}

/// A micro version of Rust's standard library's prelude.
#[macro_export]
macro_rules! tiny_std {
    () => {
        extern crate alloc;

        #[global_allocator]
        static ALLOC: $crate::alloc::QuantumHeap = $crate::alloc::QuantumHeap::new();

        #[cfg(not(test))]
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::dbugln!("{}", info);
            $crate::exit($crate::ExitReason::Failure);
        }

        #[doc(hidden)]
        mod hidden_debug {
            pub(super) fn debug_output(args: ::core::fmt::Arguments) {
                use core::fmt::Write;
                _ = (::libq::debug::DebugOut {}).write_fmt(args);
            }
        }

        #[unsafe(link_section = ".start")]
        #[unsafe(no_mangle)]
        extern "C" fn _start() {
            ::libq::debug::set_global_debug_fn(hidden_debug::debug_output);

            let main_result = main();
            let exit_status = $crate::QuantumTermination::exit_status(main_result);

            $crate::exit(exit_status);
        }
    };
}
