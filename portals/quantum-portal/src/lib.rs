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

use portal::portal2;

#[portal2(protocol = "syscall", global = true)]
pub trait QuantumPortal {
    #[event = 0]
    fn exit(exit_reason: ExitReason) -> ! {
        enum ExitReason {
            Success,
            // TODO: Failure should maybe take an Error of some sort
            //       to propagate errors from one process to another
            Failure,
        }
    }

    #[event = 1]
    fn map_memory(
        location: MemoryLocation,
        protections: MemoryProtections,
        bytes: usize,
    ) -> Result<*mut u8, MapMemoryError> {
        enum MemoryLocation {
            Anywhere,
        }
        enum MemoryProtections {
            ReadOnly,
            ReadWrite,
            ReadExecute,
            None,
        }
        enum MapMemoryError {
            InvalidLength(usize),
            OutOfMemory,
            MappingMemoryError,
        }
    }

    #[event = 2]
    fn get_pid() -> usize;

    #[event = 3]
    fn wait_for(conditions: &[WaitCondition], signal_buffer: &mut [WaitSignal]) -> Result<usize, WaitingError> {
        enum WaitCondition {
            /// Wait for this handle to be ready to write
            WaitToWrite(u64),
            /// Wait for this handle to have bytes ready to read.
            WaitToRead(u64),
            /// Waits for any update
            WaitAny(u64),
            /// Sleep the process
            SleepMs(u64),
        }

        enum WaitSignal {
            /// Updates for handles
            HandleUpdate {
                kind: HandleUpdateKind,
                handle: u64,
                wait_index: usize,
            },
            /// Updates for sleep
            SleepUpdate {
                sleep_ms_duration: u64,
                wait_index: usize,
            },
            /// Your process is requested to exit
            TerminationRequest,
        }

        enum HandleUpdateKind {
            /// This handle is ready for data to be written
            WriteReady,
            /// This handle is ready to read, and has bytes in que
            ReadReady,
            /// This handle has disconnected
            Disconnected,
            /// This handle has accepted a new connection
            NewConnection { new_handle: u64 },
        }

        enum WaitingError {
            UnknownHandle {
                handle: u64,
                array_index: usize
            },
            
        }
    }

    #[event = 69]
    fn debug_msg(msg: &str) -> Result<(), DebugMsgError> {
        enum DebugMsgError {
            InvalidPtr(*const u8),
            InvalidLength(usize),
        }
    }
}
