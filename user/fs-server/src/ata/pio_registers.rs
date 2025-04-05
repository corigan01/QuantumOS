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

use aloe::uio::{CpuIO, UserIO, opt};

type IoRw<OwnKind> = UserIO<CpuIO, opt::ReadWrite, OwnKind>;
type IoRo<OwnKind> = UserIO<CpuIO, opt::ReadOnly, OwnKind>;
type IoWo<OwnKind> = UserIO<CpuIO, opt::WriteOnly, OwnKind>;

pub struct DataReg(IoRw<opt::Shared>);
pub struct ErrorReg(IoRo<opt::Shared>);
pub struct FeaturesReg(IoWo<opt::Shared>);
pub struct SectorCountReg(IoRw<opt::Owned>);
pub struct SectorNumberRegs {
    lo: IoRw<opt::Owned>,
    mi: IoRw<opt::Owned>,
    hi: IoRw<opt::Owned>,
}
pub struct DriveReg(IoRw<opt::Owned>);
pub struct StatusReg(IoRo<opt::Shared>);
pub struct CommandReg(IoWo<opt::Shared>);
pub struct AltStatusReg(IoRo<opt::Shared>);
pub struct DeviceControlReg(IoWo<opt::Shared>);
pub struct DriveAddressReg(IoRo<opt::Shared>);

impl DataReg {
    pub fn new(port: IoRw<opt::Shared>) -> Self {
        Self(port)
    }

    pub unsafe fn read(&self) -> u16 {
        unsafe { self.0.read_u16() }
    }

    pub unsafe fn write(&mut self, value: u16) {
        unsafe { self.0.write_u16(value) };
    }
}

#[derive(Clone, Copy)]
pub struct ErrorValue(u8);

impl ErrorValue {
    pub const AMNF_BIT: u8 = 0;
    pub const TKZNF_BIT: u8 = 1;
    pub const ABRT_BIT: u8 = 2;
    pub const MCR_BIT: u8 = 3;
    pub const IDNF_BIT: u8 = 4;
    pub const MC_BIT: u8 = 5;
    pub const UNC_BIT: u8 = 6;
    pub const BBK_BIT: u8 = 7;

    pub const fn any_error(&self) -> bool {
        self.0 != 0
    }
}

impl ErrorReg {
    pub fn new(port: IoRo<opt::Shared>) -> Self {
        Self(port)
    }

    pub unsafe fn read_lba28(&self) -> ErrorValue {
        unsafe { ErrorValue(self.0.read_u8()) }
    }

    pub unsafe fn read_lba48(&self) -> ErrorValue {
        unsafe { ErrorValue(self.0.read_u16() as u8) }
    }
}

impl FeaturesReg {
    pub fn new(port: IoWo<opt::Shared>) -> Self {
        Self(port)
    }
}
