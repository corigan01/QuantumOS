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

use crate::addr::{AlignedTo, AlignmentError, AlignmentTo, PhysAddr, VirtAddr};
use core::marker::PhantomData;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Sub;
use core::ops::SubAssign;
use util::consts::PAGE_4K;

pub trait PagingStructureSize: Clone + Copy {
    const N_PAGES: usize;
    const N_BYTES: usize = Self::N_PAGES * PAGE_4K;
}

#[derive(Clone, Copy)]
pub struct Page4K {}
#[derive(Clone, Copy)]
pub struct Page2M {}
#[derive(Clone, Copy)]
pub struct Page1G {}

impl PagingStructureSize for Page4K {
    const N_PAGES: usize = 1;
}
impl PagingStructureSize for Page2M {
    const N_PAGES: usize = 512;
}
impl PagingStructureSize for Page1G {
    const N_PAGES: usize = 262144;
}

/// An error for an alignment operation.
pub struct PageAlignmentError<const REQUIRED_PAGE_ALIGNMENT: usize> {
    addr: usize,
}

impl<const REQUIRED_PAGE_ALIGNMENT: usize> core::fmt::Debug
    for PageAlignmentError<REQUIRED_PAGE_ALIGNMENT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageAlignmentError")
            .field("alignment", &(REQUIRED_PAGE_ALIGNMENT * PAGE_4K))
            .field("addr", &self.addr)
            .finish()
    }
}

/// A structure representing a well aligned Physical page
#[derive(Clone, Copy)]
pub struct PhysPage<Size: PagingStructureSize = Page4K> {
    id: usize,
    _ph: PhantomData<Size>,
}

// FIXME: It would be nice to make 'From' for types that we know are aligned, however, Rust's type-system
//        prevents us from having multiple defs for PhysAddr<T>. Maybe this is possible in the future?
impl<AnyAlignment: AlignmentTo> TryFrom<PhysAddr<AnyAlignment>> for PhysPage<Page4K> {
    type Error = PageAlignmentError<PAGE_4K>;

    fn try_from(value: PhysAddr<AnyAlignment>) -> Result<Self, Self::Error> {
        if value.is_aligned_to(PAGE_4K) {
            Ok(Self::new(value.addr() / PAGE_4K))
        } else {
            Err(PageAlignmentError { addr: value.addr() })
        }
    }
}

impl<S: PagingStructureSize, const ALIGNMENT: usize> TryFrom<PhysPage<S>>
    for PhysAddr<AlignedTo<ALIGNMENT>>
{
    type Error = AlignmentError<ALIGNMENT>;

    fn try_from(value: PhysPage<S>) -> Result<Self, Self::Error> {
        value.addr().try_into()
    }
}

impl<S: PagingStructureSize> From<PhysPage<S>> for PhysAddr {
    fn from(value: PhysPage<S>) -> Self {
        PhysAddr::from(value.addr())
    }
}

impl<S: PagingStructureSize> PhysPage<S> {
    /// Make a new physical page from the page's ID
    ///
    /// # Note
    /// This is based on the size of the page, if the page is of Page2M then
    /// a page ID of 2 will be Page4K's 1024-th page.
    pub const fn new(id: usize) -> Self {
        Self {
            id,
            _ph: PhantomData,
        }
    }

    /// Get the current page sized value that would contain the given address.
    pub const fn containing_addr<Alignment: AlignmentTo>(addr: PhysAddr<Alignment>) -> Self {
        Self::new(addr.addr() / S::N_BYTES)
    }

    /// Get the address that this page represents.
    pub const fn addr(&self) -> PhysAddr {
        // We know this is safe because we are of this alignment
        unsafe { PhysAddr::new_unchecked(self.id * S::N_BYTES) }
    }

    /// Check if the paging alignment of ourself is the same as the expected alignment of `TheirSize`.
    pub const fn is_aligned_to<TheirSize: PagingStructureSize>(&self) -> bool {
        (self.id * S::N_PAGES) % TheirSize::N_PAGES == 0
    }

    /// Get the distance (in bytes) from `self` to `lhs`.
    ///
    /// # Note
    /// This function uses `lhs` as the end point, and will panic if `lhs` is less than `self`.
    pub const fn distance_to(&self, lhs: &Self) -> usize {
        (lhs.id - self.id) * S::N_BYTES
    }

    /// Get the distance (in pages) from `self` to `lhs`.
    ///
    /// # Note
    ///  - This function uses `lhs` as the end point, and will panic if `lhs` is less than `self`.
    ///  - This function is also 'sized' to the current page size of `self`.
    pub const fn pages_to(&self, lhs: &Self) -> usize {
        lhs.id - self.id
    }

    /// Get the page id of this page.
    pub const fn page(&self) -> usize {
        self.id
    }
}

/// A structure representing a well aligned Virtual page
pub struct VirtPage<Size: PagingStructureSize = Page4K> {
    id: usize,
    _ph: PhantomData<Size>,
}

// FIXME: It would be nice to make 'From' for types that we know are aligned, however, Rust's type-system
//        prevents us from having multiple defs for VirtAddr<T>. Maybe this is possible in the future?
impl<AnyAlignment: AlignmentTo> TryFrom<VirtAddr<AnyAlignment>> for VirtPage<Page4K> {
    type Error = PageAlignmentError<PAGE_4K>;

    fn try_from(value: VirtAddr<AnyAlignment>) -> Result<Self, Self::Error> {
        if value.is_aligned_to(PAGE_4K) {
            Ok(Self::new(value.addr() / PAGE_4K))
        } else {
            Err(PageAlignmentError { addr: value.addr() })
        }
    }
}

impl<S: PagingStructureSize, const ALIGNMENT: usize> TryFrom<VirtPage<S>>
    for VirtAddr<AlignedTo<ALIGNMENT>>
{
    type Error = AlignmentError<ALIGNMENT>;

    fn try_from(value: VirtPage<S>) -> Result<Self, Self::Error> {
        value.addr().try_into()
    }
}

impl<S: PagingStructureSize> From<VirtPage<S>> for VirtAddr {
    fn from(value: VirtPage<S>) -> Self {
        VirtAddr::from(value.addr())
    }
}

impl<S: PagingStructureSize> VirtPage<S> {
    /// Make a new virtual page from the page's ID
    ///
    /// # Note
    /// This is based on the size of the page, if the page is of Page2M then
    /// a page ID of 2 will be Page4K's 1024-th page.
    pub const fn new(id: usize) -> Self {
        Self {
            id,
            _ph: PhantomData,
        }
    }

    /// Get the current page sized value that would contain the given address.
    pub const fn containing_addr<Alignment: AlignmentTo>(addr: VirtAddr<Alignment>) -> Self {
        Self::new(addr.addr() / S::N_BYTES)
    }

    /// Get the address that this page represents.
    #[inline]
    pub const fn addr(&self) -> VirtAddr {
        // We know this is safe because we are of this alignment
        unsafe { VirtAddr::new_unchecked(self.id * S::N_BYTES) }
    }

    /// Check if the paging alignment of ourself is the same as the expected alignment of `TheirSize`.
    pub const fn is_aligned_to<TheirSize: PagingStructureSize>(&self) -> bool {
        (self.id * S::N_PAGES) % TheirSize::N_PAGES == 0
    }

    /// Get the distance (in bytes) from `self` to `lhs`.
    ///
    /// # Note
    /// This function uses `lhs` as the end point, and will panic if `lhs` is less than `self`.
    pub const fn distance_to(&self, lhs: &Self) -> usize {
        (lhs.id - self.id) * S::N_BYTES
    }

    /// Get the distance (in pages) from `self` to `lhs`.
    ///
    /// # Note
    ///  - This function uses `lhs` as the end point, and will panic if `lhs` is less than `self`.
    ///  - This function is also 'sized' to the current page size of `self`.
    pub const fn pages_to(&self, lhs: &Self) -> usize {
        lhs.id - self.id
    }

    /// Get the page id of this page.
    pub const fn page(&self) -> usize {
        self.id
    }
}

macro_rules! impl_traits_for {
    ($($t:ty),*) => {
        $(
            impl PartialEq for $t {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }

            impl Eq for $t {}

            impl Ord for $t {
                fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                    self.id.cmp(&other.id)
                }
            }

            impl PartialOrd for $t {
                fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                    self.id.partial_cmp(&other.id)
                }
            }

            impl core::fmt::Debug for $t {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_tuple(stringify!($t))
                        .field(&self.id)
                        .finish()
                }
            }

            impl Sub for $t {
                type Output = $t;

                fn sub(self, rhs: Self) -> Self::Output {
                    Self::new(self.page() - rhs.page())
                }
            }

            impl SubAssign for $t{
                fn sub_assign(&mut self, rhs: Self) {
                    self.id -= rhs.id;
                }
            }

            impl Add for $t  {
                type Output = $t;

                fn add(self, rhs: Self) -> Self::Output {
                    Self::new(self.page() + rhs.page())
                }
            }

            impl AddAssign for $t {
                fn add_assign(&mut self, rhs: Self) {
                    self.id -= rhs.id;
                }
            }

        )*
    };
}

impl_traits_for! { PhysPage<Page4K>, PhysPage<Page2M>, PhysPage<Page1G>, VirtPage<Page4K>, VirtPage<Page2M>, VirtPage<Page1G> }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_page_into_phys_page() {
        let page: PhysPage<Page4K> = PhysPage::new(1);

        assert_eq!(page.addr(), PhysAddr::from(4096));
    }
}
