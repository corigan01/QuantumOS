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

use portal::portal;

#[portal(protocol = "syscall", global = true)]
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

    /// Map a memory region into this process's memory map
    ///
    /// This function acts like mmap on common unix systems.
    #[event = 1]
    fn map_memory(
        location: MemoryLocation,
        protections: MemoryProtections,
        bytes: usize,
    ) -> Result<*mut u8, MapMemoryError> {
        enum MemoryLocation {
            Anywhere,
            PhysicalLoc(u64),
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
    fn signal_wait() -> WaitSignal {
        enum WaitSignal {
            /// Updates for handles
            HandleUpdate { kind: HandleUpdateKind, handle: u64 },
            /// Updates for sleep
            TimerUpdate { ms_duration: u64 },
            /// Your process is requested to exit
            TerminationRequest,
            /// There is no condition in this slot
            None,
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
    }

    #[event = 4]
    fn yield_now() {}

    /// Receive data from a handle
    #[event = 5]
    fn recv(handle: u64, buf: &mut [u8]) -> Result<usize, RecvHandleError> {
        enum RecvHandleError {
            InvalidHandle,
            RecvFailed,
            WouldBlock,
        }
    }

    /// Send data to a handle
    #[event = 6]
    fn send(handle: u64, buf: &[u8]) -> Result<usize, SendHandleError> {
        enum SendHandleError {
            InvalidHandle,
            SendFailed,
            WouldBlock,
        }
    }

    #[event = 7]
    fn serve(endpoint: &str) -> Result<u64, ServeHandleError> {
        enum ServeHandleError {
            AlreadyBound,
        }
    }

    #[event = 8]
    fn connect(endpoint: &str) -> Result<u64, ConnectHandleError> {
        enum ConnectHandleError {
            EndpointDoesNotExist,
        }
    }

    /// Disconnect the handle if one exists
    #[event = 9]
    fn close(handle: u64) {}

    /// Unmap a memory region allocated with [`map_memory`]
    #[event = 10]
    fn unmap_memory(ptr: *mut u8) {}

    #[event = 11]
    unsafe fn fixme_cpuio_read_u8(address: u16) -> u8 {}

    #[event = 12]
    unsafe fn fixme_cpuio_write_u8(address: u16, data: u8) {}

    #[event = 13]
    unsafe fn fixme_cpuio_read_u16(address: u16) -> u16 {}

    #[event = 14]
    unsafe fn fixme_cpuio_write_u16(address: u16, data: u16) {}

    #[event = 69]
    fn debug_msg(msg: &str) -> Result<(), DebugMsgError> {
        enum DebugMsgError {
            InvalidPtr(*const u8),
            InvalidLength(usize),
        }
    }
}
