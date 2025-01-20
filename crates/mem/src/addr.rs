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

use core::marker::PhantomData;

/// A trait to enfore structs to be aligned
pub trait AlignmentTo: Clone + Copy {}

/// A non-aligned aligned ptr
#[derive(Clone, Copy)]
pub struct NotAligned {}
/// A aligned ptr
#[derive(Clone, Copy)]
pub struct AlignedTo<const ALIGNMENT: usize> {}

impl AlignmentTo for NotAligned {}
impl<const ALIGNMENT: usize> AlignmentTo for AlignedTo<ALIGNMENT> {}

/// An error for an alignment operation.
pub struct AlignmentError<const ALIGNMENT: usize>(());

impl<const ALIGNMENT: usize> core::fmt::Debug for AlignmentError<ALIGNMENT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("AlignmentError").field(&ALIGNMENT).finish()
    }
}

macro_rules! make_addr {
    (
        $(#[$attr:meta])*
        $ident:ident
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy)]
        pub struct $ident<A: AlignmentTo = NotAligned> {
            addr: usize,
            _ph: PhantomData<A>,
        }

        impl<T: AlignmentTo> PartialEq for $ident<T> {
            fn eq(&self, other: &Self) -> bool {
                self.addr == other.addr
            }
        }

        impl<T: AlignmentTo> PartialOrd for $ident<T> {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.addr.cmp(&other.addr))
            }
        }

        impl<T: AlignmentTo> Ord for $ident<T> {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.addr.cmp(&other.addr)
            }
        }

        impl<T: AlignmentTo> Eq for $ident<T> {}

        impl core::fmt::Debug for $ident<NotAligned> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($ident)).field(&self.addr).finish()
            }
        }

        impl<const ALIGNMENT: usize> core::fmt::Debug for $ident<AlignedTo<ALIGNMENT>> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(concat!(stringify!($ident), "<Aligned>"))
                    .field("align", &ALIGNMENT)
                    .field("addr", &self.addr)
                    .finish()
            }
        }

        impl<T: AlignmentTo> core::fmt::Display for $ident<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.addr.fmt(f)
            }
        }

        impl<T: AlignmentTo> core::fmt::LowerHex for $ident<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.addr.fmt(f)
            }
        }

        impl<T: AlignmentTo> core::fmt::UpperHex for $ident<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.addr.fmt(f)
            }
        }

        impl<T> From<*const T> for $ident<NotAligned> {
            fn from(value: *const T) -> Self {
                Self::new(value.addr())
            }
        }

        impl<T> From<&T> for $ident<NotAligned> {
            fn from(value: &T) -> Self {
                Self::new(value as *const _ as usize)
            }
        }

        impl<T> From<&mut T> for $ident<NotAligned> {
            fn from(value: &mut T) -> Self {
                Self::new(value as *const _ as usize)
            }
        }

        impl From<usize> for $ident<NotAligned> {
            fn from(value: usize) -> Self {
                Self::new(value)
            }
        }

        impl<T, const ALIGNMENT: usize> TryFrom<*const T> for $ident<AlignedTo<ALIGNMENT>> {
            type Error = AlignmentError<ALIGNMENT>;

            fn try_from(value: *const T) -> Result<Self, Self::Error> {
                if !ALIGNMENT.is_power_of_two() {
                    panic!("Alignment {} should be a power of 2!", ALIGNMENT);
                }

                if value.addr() & (ALIGNMENT - 1) == 0 {
                    Ok(Self::try_new(value.addr()))
                } else {
                    Err(AlignmentError(()))
                }
            }
        }

        impl<const ALIGNMENT: usize> TryFrom<usize> for $ident<AlignedTo<ALIGNMENT>> {
            type Error = AlignmentError<ALIGNMENT>;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                if !ALIGNMENT.is_power_of_two() {
                    panic!("Alignment {} should be a power of 2!", ALIGNMENT);
                }

                if value & (ALIGNMENT - 1) == 0 {
                    Ok(Self::try_new(value))
                } else {
                    Err(AlignmentError(()))
                }
            }
        }

        impl<const ALIGNMENT: usize> TryFrom<$ident<NotAligned>> for $ident<AlignedTo<ALIGNMENT>> {
            type Error = AlignmentError<ALIGNMENT>;

            fn try_from(value: $ident<NotAligned>) -> Result<Self, Self::Error> {
                if !ALIGNMENT.is_power_of_two() {
                    panic!("Alignment {} should be a power of 2!", ALIGNMENT);
                }

                if value.addr() & (ALIGNMENT - 1) == 0 {
                    Ok(Self::try_new(value.addr()))
                } else {
                    Err(AlignmentError(()))
                }
            }
        }

        impl<A: AlignmentTo> $ident<A> {
            /// Force a value as an addr.
            #[inline]
            pub const unsafe fn new_unchecked(value: usize) -> Self {
               Self {
                  addr: value,
                  _ph: PhantomData
               }
            }

            /// Get the address contained within the $ident
            pub const fn addr(&self) -> usize {
                self.addr
            }

            /// Cast the inner addr into a ptr
            pub const fn as_ptr<T>(&self) -> *const T {
                self.addr as *const T
            }

            /// Cast the inner addr into a ptr
            pub const fn as_mut_ptr<T>(&self) -> *mut T {
                self.addr as *mut T
            }

            /// Align the addr by bumping up its value until it reaches a valid alignment.
            pub const fn align_up_to(mut self, alignment: usize) -> Self {
                let rmd = self.addr % alignment;

                self.addr = if rmd != 0 {
                    alignment - rmd + self.addr
                } else {
                    self.addr
                };

                self
            }

            /// Align the addr by bumping down its value until it reaches a valid alignment.
            pub const fn align_down_to(mut self, alignment: usize) -> Self {
                let rmd = self.addr % alignment;
                self.addr = if rmd != 0 { self.addr - rmd } else { self.addr };

                self
            }

            /// Align up this ptr to fit nicely into the aligned type.
            ///
            /// # Note
            /// This will shift the ptr up until it meets the alignment.
            pub const fn align_into<const ALIGNMENT: usize>(self) -> $ident<AlignedTo<ALIGNMENT>> {
                let rmd = self.addr % ALIGNMENT;

                $ident {
                    addr: if rmd != 0 {
                        ALIGNMENT - rmd + self.addr
                    } else {
                        self.addr
                    },
                    _ph: PhantomData,
                }
            }

            /// Check if this ptr is null.
            pub const fn is_null(&self) -> bool {
                self.addr == 0
            }


            /// Make a new ptr to nothing.
            pub const fn dangling() -> Self {
                Self {
                    addr: 0,
                    _ph: PhantomData
                }
            }

            /// Get the length from `self` to `ptr`.
            ///
            /// # Note
            /// `ptr` is used as the 'end' address in this calculation. This function
            /// will panic if `ptr` is less than `self`
            pub const fn distance_to<U: AlignmentTo>(&self, ptr: PhysAddr<U>) -> usize {
                ptr.addr - self.addr
            }

            /// Check if this addr is aligned to some Alignment
            pub const fn is_aligned_to(&self, alignment: usize) -> bool {
                self.addr & (alignment - 1) == 0
            }

            /// Chop a component of bits from this address
            pub const fn chop_bottom(&self, alignment: usize) -> usize {
                if !alignment.is_power_of_two() {
                    panic!("Alignment should be a power of 2!");
                }

                self.addr & (alignment - 1)
            }


        }

        impl<const ALIGNMENT: usize> $ident<AlignedTo<ALIGNMENT>> {
            /// Attempt to offset this `addr` by an `offset`.
            pub const fn try_offset(self, offset: usize) -> Result<Self, AlignmentError<ALIGNMENT>> {
                if offset & (ALIGNMENT - 1) != 0 {
                    return Err(AlignmentError(()));
                }

                Ok(Self::try_new(self.addr + offset))
            }

            /// Get the 'realative' value based on the size of a chunk.
            ///
            /// This preforms `self.addr() % element_size` internally.
            ///
            /// This function asserts that the 'chunk_size' is aligned to
            /// this addresses alignment constraints.
            pub fn realative_offset(self, chunk_size: usize) -> Self {
                assert!((ALIGNMENT % chunk_size == 0) || (ALIGNMENT % chunk_size == ALIGNMENT), "'chunk_size' is not aligned to address alignment!");
                Self::try_new(self.addr() % chunk_size)
            }

            /// Make a new address, asserts if address is not aligned.
            pub const fn try_new(addr: usize) -> Self {
                assert!(addr & (ALIGNMENT - 1) == 0, "Address not aligned");
                Self {
                    addr,
                    _ph: PhantomData
                }
            }

        }

        impl $ident<NotAligned> {
            /// Make a new address.
            pub const fn new(addr: usize) -> Self {
                Self {
                    addr,
                    _ph: PhantomData
                }
            }

            /// Offset this addr by `offset`.
            pub const fn offset(self, offset: usize) -> Self {
                Self{ addr: self.addr + offset, _ph: PhantomData }
            }

            /// Get the 'realative' value based on the size of a chunk.
            ///
            /// This preforms `self.addr() % element_size` internally.
            ///
            /// This function asserts that the 'chunk_size' is aligned to
            /// this addresses alignment constraints.
            pub const fn realative_offset(self, chunk_size: usize) -> Self {
                Self {
                    addr: (self.addr() % chunk_size),
                    _ph: PhantomData
                }
            }

            /// Add to this address (in bytes)
            pub const fn extend_by(self, bytes: usize) -> Self {
                Self::new(self.addr() + bytes)
            }
        }

    };
}

make_addr! {
    /// A structure safely repr a ptr to physical memory.
    ///
    /// Can be aligned, or non aligned though generic.
    PhysAddr
}

make_addr! {
    /// A structure safely repr a ptr to Virtual memory.
    ///
    /// Can be aligned, or non aligned though generic.
    VirtAddr
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alignment_up_to() {
        let mut addr = PhysAddr::from(120);

        assert_eq!(addr.align_up_to(256), PhysAddr::from(256));
    }

    #[test]
    fn test_alignment_down_to() {
        let mut addr = PhysAddr::from(120);

        assert_eq!(addr.align_down_to(256), PhysAddr::from(0));
    }

    #[test]
    fn test_alignment_into() {
        let addr = PhysAddr::from(5);

        fn needs_aligned(ptr: PhysAddr<AlignedTo<8>>) {
            assert_eq!(ptr.addr(), 8);
        }

        needs_aligned(addr.align_into());
    }

    #[test]
    #[should_panic]
    fn test_fail_alignment() {
        let addr = PhysAddr::from(15);
        let _aligned: PhysAddr<AlignedTo<16>> = addr.try_into().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_alignment_not_power_of_two() {
        let addr = PhysAddr::from(15);
        let _aligned: Result<PhysAddr<AlignedTo<15>>, AlignmentError<15>> = addr.try_into();
    }
}
