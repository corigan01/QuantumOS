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

            impl core::fmt::Debug for $ident<NotAligned> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_tuple("$ident").field(&self.addr).finish()
                }
            }

            impl<const ALIGNMENT: usize> core::fmt::Debug for $ident<AlignedTo<ALIGNMENT>> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct("$ident<Aligned>")
                        .field("align", &ALIGNMENT)
                        .field("addr", &self.addr)
                        .finish()
                }
            }

            impl<T> From<*const T> for $ident<NotAligned> {
                fn from(value: *const T) -> Self {
                    $ident {
                        addr: value.addr(),
                        _ph: PhantomData,
                    }
                }
            }

            impl<T> From<&T> for $ident<NotAligned> {
                fn from(value: &T) -> Self {
                    $ident {
                        addr: value as *const _ as usize,
                        _ph: PhantomData,
                    }
                }
            }

            impl<T> From<&mut T> for $ident<NotAligned> {
                fn from(value: &mut T) -> Self {
                    $ident {
                        addr: value as *const _ as usize,
                        _ph: PhantomData,
                    }
                }
            }

            impl From<usize> for $ident<NotAligned> {
                fn from(value: usize) -> Self {
                    $ident {
                        addr: value,
                        _ph: PhantomData,
                    }
                }
            }

            impl<T, const ALIGNMENT: usize> TryFrom<*const T> for $ident<AlignedTo<ALIGNMENT>> {
                type Error = AlignmentError<ALIGNMENT>;

                fn try_from(value: *const T) -> Result<Self, Self::Error> {
                    if !ALIGNMENT.is_power_of_two() {
                        panic!("Alignment {} should be a power of 2!", ALIGNMENT);
                    }

                    if value.addr() & (ALIGNMENT - 1) == 0 {
                        Ok($ident {
                            addr: value.addr(),
                            _ph: PhantomData,
                        })
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
                        Ok($ident {
                            addr: value,
                            _ph: PhantomData,
                        })
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
                        Ok($ident {
                            addr: value.addr(),
                            _ph: PhantomData,
                        })
                    } else {
                        Err(AlignmentError(()))
                    }
                }
            }

            impl<A: AlignmentTo> $ident<A> {
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
                pub const fn align_up_to(&mut self, alignment: usize) -> Self {
                    let rmd = self.addr % alignment;

                    self.addr = if rmd != 0 {
                        alignment - rmd + self.addr
                    } else {
                        self.addr
                    };

                    *self
                }

                /// Align the addr by bumping down its value until it reaches a valid alignment.
                pub const fn align_down_to(&mut self, alignment: usize) -> Self {
                    let rmd = self.addr % alignment;
                    self.addr = if rmd != 0 { self.addr - rmd } else { self.addr };

                    *self
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
