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

use crate::address_utils::physical_address::PhyAddress;
use crate::address_utils::region_map::RegionMap;
use crate::address_utils::virtual_address::VirtAddress;
use crate::gfx::linear_framebuffer::LinearFramebuffer;
use crate::magic::DataProtector;

#[derive(Debug)]
pub enum KernelBootInfoErr {
    NullPtr,
    InvalidMagic,
}

// TODO: implement a bootloader callback which will tell the bootloader
//       that we successfully booted into the kernel!

pub struct KernelBootInformation {
    pub physical_regions: RegionMap<PhyAddress>,
    pub virtual_regions: RegionMap<VirtAddress>,
    pub framebuffer: LinearFramebuffer,
    magic: DataProtector,
}

impl KernelBootInformation {
    pub fn new(
        physical_regions: RegionMap<PhyAddress>,
        virtual_regions: RegionMap<VirtAddress>,
        framebuffer: LinearFramebuffer,
    ) -> Self {
        Self {
            physical_regions,
            virtual_regions,
            framebuffer,
            magic: DataProtector::new(),
        }
    }

    pub fn send_as_u64(&self) -> u64 {
        self as *const Self as u64
    }

    pub unsafe fn load_from_bootloader<'a>(
        ptr: *const KernelBootInformation,
    ) -> Result<&'a KernelBootInformation, KernelBootInfoErr> {
        if ptr as usize == 0 {
            return Err(KernelBootInfoErr::NullPtr);
        }

        let deref = &*ptr;

        if !deref.magic.is_magic_valid() {
            return Err(KernelBootInfoErr::InvalidMagic);
        }

        return Ok(deref);
    }

    pub fn get_physical_memory(&self) -> &RegionMap<PhyAddress> {
        assert!(self.magic.is_magic_valid(), "Magic failed!");
        &self.physical_regions
    }

    pub fn get_virtual_memory(&self) -> &RegionMap<VirtAddress> {
        assert!(self.magic.is_magic_valid(), "Magic failed!");
        &self.virtual_regions
    }

    pub fn get_framebuffer(&self) -> &LinearFramebuffer {
        assert!(self.magic.is_magic_valid(), "Magic failed!");
        &self.framebuffer
    }
}
