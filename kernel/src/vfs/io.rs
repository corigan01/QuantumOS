/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

use core::error::Error;
use core::fmt::{Display, Formatter};
use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use quantum_utils::human_bytes::HumanBytes;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    Unknown,
    Interrupted,
    NotSeekable,
}

pub trait IOError: Error {
    fn error_kind(&self) -> ErrorKind;
}

impl PartialEq<ErrorKind> for dyn IOError {
    fn eq(&self, other: &ErrorKind) -> bool {
        self.error_kind() == *other
    }
}

impl PartialEq<dyn IOError> for dyn IOError {
    fn eq(&self, other: &dyn IOError) -> bool {
        self.error_kind() == other.error_kind()
    }
}

impl PartialEq<dyn IOError> for ErrorKind {
    fn eq(&self, other: &dyn IOError) -> bool {
        *self == other.error_kind()
    }
}

pub type IOResult<T> = Result<T, Box<dyn IOError>>;

pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

impl SeekFrom {
    pub fn modify_pos(self, max: u64, min: u64, current: u64) -> Option<u64> {
        return match self {
            SeekFrom::Start(pos) => {
                if pos > max || pos < min {
                    return None;
                }

                Some(pos)
            }
            SeekFrom::End(pos) => {
                if pos > 0 || pos + (max + min) as i64 > 0 {
                    return None;
                }

                Some(((max as i64) + pos) as u64)
            }
            SeekFrom::Current(pos) => {
                let pos_current = pos + current as i64;
                if pos_current > max as i64 || pos_current < min as i64 {
                    return None;
                }

                Some(pos_current as u64)
            }
        };
    }
}

pub trait Seek {
    fn seek(&mut self, seek: SeekFrom) -> IOResult<u64>;

    fn rewind(&mut self) -> IOResult<()> {
        self.seek(SeekFrom::Start(0))?;

        Ok(())
    }

    fn stream_len(&mut self) -> IOResult<u64> {
        let current = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(current))?;

        Ok(len)
    }

    fn stream_position(&mut self) -> IOResult<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

// FIXME: Make closer to the actual implementation
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize>;

    fn read_exact(&mut self, buf: &mut [u8]) -> IOResult<()> {
        let mut filled = 0;

        while filled <= buf.len() - 1 {
            match self.read(&mut buf[filled..]) {
                Ok(amount) => {
                    filled += amount;
                }
                Err(e) if e.error_kind() == ErrorKind::Interrupted => {
                    filled = 0;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize>;
    fn flush(&mut self) -> IOResult<()>;

    fn write_all(&mut self, buf: &[u8]) -> IOResult<()> {
        let mut written = 0;
        while written <= buf.len() - 1 {
            match self.write(&buf[written..]) {
                Ok(amount) => {
                    written += amount;
                }
                Err(e) if *e == ErrorKind::Interrupted => {
                    written = 0;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

pub enum DiskType {
    Unknown,
    HardDisk,
    SSD,
    Emulated,
}

#[derive(Clone, Copy, Debug)]
pub enum DiskBus {
    Unknown,
    ParallelPIO,
    ParallelDMA,
    Sata,
    NVMe,
    Emulated,
}

impl Display for DiskBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = match self {
            DiskBus::Unknown => "Unknown",
            DiskBus::ParallelPIO => "PIO",
            DiskBus::ParallelDMA => "ATA",
            DiskBus::Sata => "SATA",
            DiskBus::NVMe => "NVMe",
            DiskBus::Emulated => "VirtIO"
        };

        f.write_str(str)
    }
}

pub trait DiskInfo {
    fn disk_type(&self) -> DiskType {
        DiskType::Unknown
    }

    fn disk_bus(&self) -> DiskBus {
        DiskBus::Unknown
    }

    fn disk_model(&self) -> String {
        String::from("Unknown")
    }

    fn disk_capacity(&self) -> HumanBytes {
        HumanBytes::from(0)
    }
}

pub enum PartitionType {
    Unknown,
    MBR,
}

pub trait PartitionInfo {
    fn logical_partition_start_byte(&self) -> u64;
    fn logical_partition_end_byte(&self) -> u64;
    fn partition_size(&self) -> u64 {
        self.logical_partition_end_byte() - self.logical_partition_start_byte()
    }

    fn is_bootable(&self) -> bool {
        false
    }

    fn partition_type(&self) -> PartitionType {
        PartitionType::Unknown
    }
}

