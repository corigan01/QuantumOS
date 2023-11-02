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
*
*/

/// # Disk Info Provider
/// Info about a disk, including the size, name, and machine name. Used by fs to show properties
/// about disks.
pub trait DiskInfoProvider {
    fn human_name(&mut self) -> Option<String>;
    fn machine_name(&mut self) -> Option<String>;

    fn size_bytes(&mut self) -> Option<usize>;
}

/// # Machine Status Flags
pub enum MachineStatusFlags {
    /// # Ask Disconnect
    /// Sets the status that this machine interface would like to disconnect from the host.
    AskDisconnect,
    /// # Forced Disconnect
    /// This device was yanked out and is now disconnected. (Not ejecting a drive would set this
    /// status)
    ForcedDisconnect,
    /// # Not Responding
    /// The machine is not responding to commands, or has a child who is not responding.
    NotResponding,
    /// # Ask Connect
    /// Asks for connection.
    AskConnect,
    /// # Ready
    /// This machine is ready for commands
    Ready,
}

/// # Machine Status Field Flags
/// The flags that machine status info can return.
pub struct MachineStatusFieldFlags(u64);

/// # Machine Status Info
/// Gather infomation about the machine.
pub trait MachineStatusInfo {}
