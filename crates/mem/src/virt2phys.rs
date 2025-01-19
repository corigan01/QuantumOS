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

extern crate alloc;
use spin::RwLock;

use crate::{
    addr::{PhysAddr, VirtAddr},
    page::PhysPage,
};

#[derive(Clone, Copy, Debug)]
pub enum PhysPtrTranslationError {
    VirtNotFound(VirtAddr),
    PageEntriesNotSetup,
}

pub trait ObtainVirtAddr {
    fn virt_addr(&self) -> VirtAddr;
}

pub trait ObtainPhysAddr: ObtainVirtAddr {
    fn phys_addr(&self) -> Result<PhysAddr, PhysPtrTranslationError> {
        virt2phys(self.virt_addr())
    }
    fn phys_page(&self) -> Result<PhysPage, PhysPtrTranslationError> {
        Ok(PhysPage::containing_addr(self.phys_addr()?))
    }
}

static VIRTUAL_LOOKUP_FN: RwLock<LookupFn> = RwLock::new(no_lookup_fn);

/// This is the default ptr lookup fn that will just fail when called
fn no_lookup_fn(_ptr: VirtAddr) -> Result<PhysAddr, PhysPtrTranslationError> {
    Err(PhysPtrTranslationError::PageEntriesNotSetup)
}

/// Set the global lookup function to the provided function
pub fn set_global_lookup_fn(fun: LookupFn) {
    *VIRTUAL_LOOKUP_FN.write() = fun;
}

/// Preform virt2phys translation
pub fn virt2phys(phy_addr: VirtAddr) -> Result<PhysAddr, PhysPtrTranslationError> {
    let fun = VIRTUAL_LOOKUP_FN.read();
    (&*fun)(phy_addr)
}

/// The function type to do virtual address lookups
pub type LookupFn = fn(phy_addr: VirtAddr) -> Result<PhysAddr, PhysPtrTranslationError>;

impl<T: ObtainVirtAddr> ObtainPhysAddr for T {}
impl<T: Into<VirtAddr>> ObtainVirtAddr for T {
    fn virt_addr(&self) -> VirtAddr {
        self.into()
    }
}
