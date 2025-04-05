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

use core::marker::PhantomData;

use private::IoInterface;
use vera_portal::sys_client::{
    fixme_cpuio_read_u8, fixme_cpuio_read_u16, fixme_cpuio_write_u8, fixme_cpuio_write_u16,
};

mod private {
    pub trait IoInterface {
        /// Ask QuantumOS for ownership over this IO device.
        ///
        /// IO devices should never have multiple ownership, and should be held for the lifetime
        /// of the device's use. Other processes should also respect all device ownership to ensure
        /// two userspace programs cannot cause undefined driver behavior.
        ///
        /// If `complete_ownership` is set, this device must also respect ownership within the owner
        /// process. This means that no two references to the same device can exist on the system at
        /// all. Sometimes, however, interally in a process it might request to have shared ownership
        /// over the device.
        fn own(&self, complete_ownership: bool);

        /// Inform QuantumOS to release ownership over this IO device.
        ///
        /// IO devices should never have multiple ownership, and should be held for the lifetime
        /// of the device's use. Other processes should also respect all device ownership to ensure
        /// two userspace programs cannot cause undefined driver behavior.
        fn unown(&self);
    }

    impl IoInterface for super::CpuIO {
        fn own(&self, _: bool) {
            // FIXME: Add support for device ownership in the kernel
        }

        fn unown(&self) {
            // FIXME: Add support for device ownership in the kernel
        }
    }
}
pub unsafe trait IoAccessKind {}
unsafe impl IoAccessKind for opt::ReadOnly {}
unsafe impl IoAccessKind for opt::WriteOnly {}
unsafe impl IoAccessKind for opt::ReadWrite {}

pub unsafe trait OwnStrictness {
    const COMPLETE_OWNERSHIP: bool;
}
unsafe impl OwnStrictness for opt::Owned {
    const COMPLETE_OWNERSHIP: bool = true;
}
unsafe impl OwnStrictness for opt::Shared {
    const COMPLETE_OWNERSHIP: bool = false;
}

pub mod opt {
    pub struct Owned(());
    pub struct Shared(());

    pub struct ReadOnly(());
    pub struct WriteOnly(());
    pub struct ReadWrite(());
}

pub unsafe trait IoSupportsReading {}
pub unsafe trait IoSupportsWritting {}
pub unsafe trait IoSupportsReadWrite {}

unsafe impl<Interface: private::IoInterface, Owner: OwnStrictness> IoSupportsReading
    for UserIO<Interface, opt::ReadOnly, Owner>
{
}
unsafe impl<Interface: private::IoInterface, Owner: OwnStrictness> IoSupportsReading
    for UserIO<Interface, opt::ReadWrite, Owner>
{
}
unsafe impl<Interface: private::IoInterface, Owner: OwnStrictness> IoSupportsWritting
    for UserIO<Interface, opt::WriteOnly, Owner>
{
}
unsafe impl<Interface: private::IoInterface, Owner: OwnStrictness> IoSupportsWritting
    for UserIO<Interface, opt::ReadWrite, Owner>
{
}
unsafe impl<Interface: private::IoInterface, Owner: OwnStrictness> IoSupportsReadWrite
    for UserIO<Interface, opt::ReadWrite, Owner>
{
}

#[derive(Debug)]
#[repr(transparent)]
pub struct CpuIO(u16);

/// Userspace access to IO devices
///
/// # Currently Supported Interfaces
///  - [`CpuIO`] *CPU IO Port bus access*
///
/// # Why use this type?
/// `UserIO` represents an 'owned' access over some IO device on the system. This is important because future
/// processes will not be able to access this IO device until it is dropped.
///
/// This type also attempts to implment the interface to access such hardware device in the safest possible
/// way.
pub struct UserIO<
    Interface: private::IoInterface,
    Access: IoAccessKind = opt::ReadWrite,
    OwnKind: OwnStrictness = opt::Owned,
> {
    interface: Interface,
    access: PhantomData<Access>,
    own_kind: PhantomData<OwnKind>,
}

impl<Access: IoAccessKind, OwnKind: OwnStrictness> UserIO<CpuIO, Access, OwnKind> {
    /// Create a new CpuIO port for access in userspace.
    pub unsafe fn new(address: u16) -> Self {
        let cpu_io = CpuIO(address);
        cpu_io.own(OwnKind::COMPLETE_OWNERSHIP);

        Self {
            interface: cpu_io,
            access: PhantomData,
            own_kind: PhantomData,
        }
    }
}

impl<Access: IoAccessKind, OwnKind: OwnStrictness> UserIO<CpuIO, Access, OwnKind>
where
    UserIO<CpuIO, Access, OwnKind>: IoSupportsReading,
{
    #[inline]
    pub unsafe fn read_u8(&self) -> u8 {
        fixme_cpuio_read_u8(self.interface.0)
    }

    #[inline]
    pub unsafe fn read_u16(&self) -> u16 {
        fixme_cpuio_read_u16(self.interface.0)
    }
}

impl<Access: IoAccessKind, OwnKind: OwnStrictness> UserIO<CpuIO, Access, OwnKind>
where
    UserIO<CpuIO, Access, OwnKind>: IoSupportsWritting,
{
    #[inline]
    pub unsafe fn write_u8(&mut self, value: u8) {
        fixme_cpuio_write_u8(self.interface.0, value);
    }

    #[inline]
    pub unsafe fn write_u16(&mut self, value: u16) {
        fixme_cpuio_write_u16(self.interface.0, value);
    }
}
