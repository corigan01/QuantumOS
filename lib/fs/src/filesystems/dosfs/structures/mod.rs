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

use core::fmt::{Display, Formatter};
use quantum_lib::bitset::BitSet;

pub mod bpb;
pub mod bpb16;
pub mod bpb32;
pub mod fat;
pub mod fs_info;
pub mod file_directory;
pub mod long_name;

pub(crate) type Byte = u8;
pub(crate) type Word = u16;
pub(crate) type DoubleWord = u32;

pub(crate) type ClusterID = usize;

pub(crate) const MAX_CLUSTERS_FOR_FAT12: usize = 0xFF4;
pub(crate) const MAX_CLUSTERS_FOR_FAT16: usize = 0xFFF4;
pub(crate) const MAX_CLUSTERS_FOR_FAT32: usize = 0xFFFFFF4;

pub trait ExtendedBiosBlock {
    fn verify(&self) -> bool;
    fn volume_serial_number(&self) -> u32;
    fn volume_label(&self) -> &str;
    fn filesystem_string(&self) -> Option<&str>;

    fn fat_sectors(&self) -> Option<usize>;
    fn fs_info_sector(&self) -> Option<usize>;
}

#[derive(Clone, Copy, Debug)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32
}

pub struct FatTime {
    seconds: Byte,
    minutes: Byte,
    hours: Byte,
    day: Byte,
    month: Byte,
    year: Word
}

impl FatTime {
    pub fn from_fat_date(fat_date: Word) -> Self {
        Self::from_fat_time_and_date(fat_date, 0)
    }

    pub fn from_fat_time(fat_time: Word) -> Self {
        Self::from_fat_time_and_date(0, fat_time)
    }

    pub fn from_fat_time_and_date(date: Word, time: Word) -> Self {
        let seconds = time.get_bits(0..4) as Byte * 2;
        let minutes = time.get_bits(5..10) as Byte;
        let hours = time.get_bits(11..15) as Byte;

        let day = date.get_bits(0..4) as Byte;
        let month = date.get_bits(5..8) as Byte;
        let year_since_1980 = date.get_bits(9..15);

        Self {
            seconds,
            minutes,
            hours,
            day,
            month,
            year: year_since_1980 + 1980
        }
    }

    pub fn is_time_populated(&self) -> bool {
        self.seconds != 0 || self.minutes != 0 || self.hours != 0
    }

    pub fn is_date_populated(&self) -> bool {
        self.day != 0 || self.month != 0 || self.year != 0
    }
}

impl Display for FatTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let date_args = format_args!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds);
        let time_args = format_args!("{:02}/{:02}/{:04}", self.month, self.day, self.year);

        if self.is_date_populated() && self.is_time_populated() {
            f.write_fmt(format_args!("{} on {}",
                                     format_args!("{:02}/{:02}/{:04}", self.month, self.day, self.year),
                                     format_args!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds)
            ))
        } else if self.is_date_populated() {
            f.write_fmt(format_args!("{}",
                                     format_args!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds)
            ))
        } else {
            f.write_fmt(format_args!("{}",
                                     format_args!("{:02}/{:02}/{:04}", self.month, self.day, self.year)
            ))
        }
    }
}



