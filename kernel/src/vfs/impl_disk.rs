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

use qk_alloc::boxed::Box;
use qk_alloc::string::String;
use qk_alloc::vec::Vec;

pub enum MediumType {
    Unknown,
    HardDisk,
}

pub enum MediumBus {
    Unknown,
    Emulated,
    ATA,
    SATA,
    Nvme,
    Ram,
}

// FIXME: This should be a dyn error in the future
#[derive(Clone, Copy, Debug)]
pub enum MediumErr {
    DiskErr,
    NotReadable,
    NotWriteable
}

pub type MediumBox = Box<dyn Medium>;

pub trait Medium {
    fn is_writable(&self) -> bool {
        false
    }

    fn is_readable(&self) -> bool {
        false
    }

    fn disk_name(&self) -> String {
        String::from("Unnamed Medium")
    }

    fn disk_type(&self) -> MediumType {
        MediumType::Unknown
    }

    fn disk_bus(&self) -> MediumBus {
        MediumBus::Unknown
    }

    fn seek(&mut self, seek: SeekFrom);

    fn read_exact(&mut self, amount: usize) -> Result<Vec<u8>, MediumErr> {
        if !self.is_readable() {
            return Err(MediumErr::NotReadable)
        }

        self.read_exact_impl(amount)
    }

    fn write_exact(&mut self, buf: Vec<u8>) -> Result<(), MediumErr> {
        if !self.is_writable() {
            return Err(MediumErr::NotWriteable);
        }

        self.write_exact_impl(buf)
    }

    // FIXME: 'Vec<u8>' should be replaced with a custom type that
    //        handles lazy memory, and over memory-protection
    fn read_exact_impl(&mut self, amount: usize) -> Result<Vec<u8>, MediumErr>;
    fn write_exact_impl(&mut self, buf: Vec<u8>) -> Result<(), MediumErr>;

    fn read_append_into(&mut self, vec: &mut Vec<u8>, amount: usize) -> Result<(), MediumErr> {
        let read = self.read_exact(amount)?;

        for byte in read.iter() {
            vec.push(*byte);
        }

        Ok(())
    }

    // Maybe 'write_consume(vec)' ?
}

