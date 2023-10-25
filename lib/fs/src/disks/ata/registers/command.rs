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

use super::{DiskID, ResolveIOPortBusOffset, COMMAND_OFFSET_FROM_IO_BASE};

/// # Commands
/// ATA disk commands to send to the disk.
#[non_exhaustive]
pub enum Commands {
    Identify,
    ReadSectorsPIO,
    WriteSectorsPIO,
    CacheFlush,
}

/// # Command Register
/// ATA PIO Register for sending comands to the disk drive.
pub struct CommandRegister {}
impl ResolveIOPortBusOffset<COMMAND_OFFSET_FROM_IO_BASE> for CommandRegister {}

impl CommandRegister {
    const ATA_CMD_READ_PIO: u8 = 0x20;
    const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
    const ATA_CMD_READ_DMA: u8 = 0xC8;
    const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
    const ATA_CMD_WRITE_PIO: u8 = 0x30;
    const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
    const ATA_CMD_WRITE_DMA: u8 = 0xCA;
    const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
    const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;
    const ATA_CMD_CACHE_FLUSH_EXT: u8 = 0xEA;
    const ATA_CMD_PACKET: u8 = 0xA0;
    const ATA_IDENTIFY_PACKET: u8 = 0xA1;
    const ATA_IDENTIFY: u8 = 0xEC;

    fn resolve_command(command: Commands) -> u8 {
        #[allow(unreachable_patterns)]
        match command {
            Commands::Identify => Self::ATA_IDENTIFY,
            Commands::ReadSectorsPIO => Self::ATA_CMD_READ_PIO,
            Commands::WriteSectorsPIO => Self::ATA_CMD_WRITE_PIO,
            Commands::CacheFlush => Self::ATA_CMD_CACHE_FLUSH,

            _ => unimplemented!("ATA-DISK PIO: Command Not Implemented!"),
        }
    }

    pub fn send_command(device: DiskID, command: Commands) {
        let my_io = Self::bus_io(device);
        let command_number = Self::resolve_command(command);

        unsafe { my_io.write_u8(command_number) }
    }
}
