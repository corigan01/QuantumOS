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

use util::bytes::HumanBytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhysMemoryKind {
    None,
    Free,
    Reserved,
    Special,
    AcpiReclaimable,
    Bootloader,
    Kernel,
    InitFs,
    PageTables,
    Broken,
}

pub trait MemoryDesc {
    fn memory_kind(&self) -> PhysMemoryKind;
    fn memory_start(&self) -> u64;
    fn memory_end(&self) -> u64;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PhysMemoryBorder {
    kind: PhysMemoryKind,
    address: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysMemoryEntry {
    pub kind: PhysMemoryKind,
    pub start: u64,
    pub end: u64,
}

impl MemoryDesc for PhysMemoryEntry {
    fn memory_kind(&self) -> PhysMemoryKind {
        self.kind
    }

    fn memory_start(&self) -> u64 {
        self.start
    }

    fn memory_end(&self) -> u64 {
        self.end
    }
}

impl MemoryDesc for &PhysMemoryEntry {
    fn memory_kind(&self) -> PhysMemoryKind {
        self.kind
    }

    fn memory_start(&self) -> u64 {
        self.start
    }

    fn memory_end(&self) -> u64 {
        self.end
    }
}

impl PhysMemoryEntry {
    pub const fn empty() -> Self {
        Self {
            kind: PhysMemoryKind::None,
            start: 0,
            end: 0,
        }
    }

    pub const fn len(&self) -> u64 {
        self.end - self.start
    }

    /// Write a pattern of bytes to this area
    ///
    /// # Note
    /// The pages that repr this memory entry must already be writeable and page mapped!
    pub unsafe fn scrub(&self, byte_pattern: u8) {
        let phys_slice = unsafe {
            core::slice::from_raw_parts_mut(self.start as *mut u8, (self.end - self.start) as usize)
        };
        phys_slice.fill(byte_pattern);
    }
}

pub struct PhysMemoryIter<'a, const N: usize> {
    mm: &'a PhysMemoryMap<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for PhysMemoryIter<'a, N> {
    type Item = PhysMemoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.mm.len {
                break None;
            }

            let first = self.mm.borders.get(self.index)?;
            let second = self.mm.borders.get(self.index + 1)?;

            self.index += 1;

            if first.kind == PhysMemoryKind::None {
                continue;
            }

            break Some(PhysMemoryEntry {
                kind: first.kind,
                start: first.address,
                end: second.address,
            });
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhysMemoryMap<const N: usize> {
    borders: [PhysMemoryBorder; N],
    len: usize,
}

impl<const N: usize> PhysMemoryMap<N> {
    pub const fn new() -> Self {
        Self {
            borders: [PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0,
            }; N],
            len: 0,
        }
    }

    pub fn iter(&self) -> PhysMemoryIter<'_, N> {
        PhysMemoryIter { mm: self, index: 0 }
    }

    pub fn bytes_of(&self, kind: PhysMemoryKind) -> usize {
        let mut bytes = 0;
        self.iter()
            .filter(|r| r.kind == kind)
            .for_each(|region| bytes += region.end - region.start);

        bytes as usize
    }

    pub fn sdram_bytes(&self) -> usize {
        let mut bytes = 0;
        self.iter()
            .filter(|r| match r.kind {
                PhysMemoryKind::Free
                | PhysMemoryKind::AcpiReclaimable
                | PhysMemoryKind::Bootloader
                | PhysMemoryKind::Kernel
                | PhysMemoryKind::PageTables => true,
                _ => false,
            })
            .for_each(|region| bytes += region.end - region.start);

        bytes as usize
    }

    pub fn find_continuous_of(
        &mut self,
        from_kind: PhysMemoryKind,
        bytes: usize,
        alignment: usize,
        min_address: u64,
    ) -> Option<PhysMemoryEntry> {
        self.iter()
            .filter(|region| region.kind == from_kind)
            .find_map(|region| {
                let new_start = util::align_to(region.start.max(min_address), alignment);

                if region.end > new_start && (region.end - new_start) >= bytes as u64 {
                    Some(PhysMemoryEntry {
                        kind: region.kind,
                        start: new_start,
                        end: new_start + bytes as u64,
                    })
                } else {
                    None
                }
            })
    }

    pub fn add_region(&mut self, region: impl MemoryDesc) -> Result<(), crate::MemoryError> {
        let kind = region.memory_kind();
        let start = region.memory_start();
        let end = region.memory_end();

        // If its a None, we don't need to do any work
        if kind == PhysMemoryKind::None {
            return Ok(());
        }

        if start >= end {
            return Err(crate::MemoryError::EntrySizeIsNegative);
        }

        let Some(s_index) = self.borders[..self.len]
            .iter()
            .enumerate()
            .filter(|(i, bor)| {
                (*i == 0 && bor.address >= start && bor.address < end)
                    || (*i != 0
                        && self.borders[*i].address >= start
                        && self.borders.get(i - 1).is_some_and(|b| b.address < start))
            })
            .last()
            .map(|(i, _)| i)
            .and_then(|ideal| {
                self.borders
                    .iter()
                    .enumerate()
                    .skip(ideal)
                    .take_while(|(i, bor)| {
                        bor.address < end
                            || (*i != 0
                                && self
                                    .borders
                                    .get(i - 1)
                                    .is_some_and(|b| b.address < start && b.kind < kind))
                    })
                    .find(|(_, bor)| bor.kind <= kind)
                    .map(|(i, _)| i)
            })
            .or_else(|| {
                if self.len == 0 {
                    Some(0)
                } else if self.borders[self.len - 1].address < start
                    && self.borders[self.len - 1].kind < kind
                {
                    Some(self.len)
                } else {
                    None
                }
            })
        else {
            return Ok(());
        };

        if s_index >= self.len {
            self.borders[s_index].kind = kind;
            self.borders[s_index].address = start;
            self.len += 1;
        } else {
            self.insert_raw(s_index, PhysMemoryBorder {
                kind,
                address: start,
            })?;
        }

        let (mut should_insert, mut last_remove_kind) = if s_index > 0 {
            self.borders
                .get(s_index - 1)
                .and_then(|bor| {
                    if bor.kind < kind {
                        Some((true, bor.kind))
                    } else {
                        None
                    }
                })
                .unwrap_or((false, PhysMemoryKind::None))
        } else {
            (false, PhysMemoryKind::None)
        };
        let mut last_larger = false;
        let mut i = s_index + 1;

        while i < self.borders.len() {
            if i >= self.len {
                should_insert = true;
                break;
            }

            if self.borders[i].address > end {
                break;
            }

            if self.borders[i].kind > kind {
                last_larger = true;
                i += 1;
                continue;
            }

            if last_larger {
                self.borders[i].kind = kind;
                should_insert = false;
                i += 1;
                continue;
            }

            last_remove_kind = self.borders[i].kind;
            self.remove_raw(i)?;
            should_insert = true;
        }

        if should_insert {
            self.insert_raw(i, PhysMemoryBorder {
                kind: last_remove_kind,
                address: end,
            })?;
        }

        Ok(())
    }

    fn insert_raw(
        &mut self,
        index: usize,
        border: PhysMemoryBorder,
    ) -> Result<(), crate::MemoryError> {
        if self.len == self.borders.len() {
            return Err(crate::MemoryError::ArrayTooSmall);
        }

        if index > self.len {
            return Err(crate::MemoryError::InvalidSize);
        }

        // if the index is at the end of the array, we don't need to move any
        // of the elements.
        if index == self.len {
            self.borders[index] = border;
            self.len += 1;
            return Ok(());
        }

        let border_len = self.borders.len() - 1;
        self.borders.copy_within(index..border_len, index + 1);
        self.borders[index] = border;
        self.len += 1;

        Ok(())
    }

    fn remove_raw(&mut self, index: usize) -> Result<(), crate::MemoryError> {
        if index >= self.len {
            return Err(crate::MemoryError::InvalidSize);
        }

        // if the index is at the end of the array, we don't need to move any
        // of the elements.
        if index + 1 == self.len {
            self.len -= 1;
            self.borders[index].kind = PhysMemoryKind::None;
            self.borders[index].address = 0;
            return Ok(());
        }

        let border_len = self.borders.len();
        self.borders.copy_within(index + 1..border_len, index);
        self.len -= 1;

        self.borders[self.len].kind = PhysMemoryKind::None;
        self.borders[self.len].address = 0;

        Ok(())
    }
}

impl<const N: usize> core::fmt::Display for PhysMemoryMap<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            "|     Kind     |       Start      |        End       |        Size       |\n",
        )?;
        f.write_str(
            "+--------------+------------------+------------------+-------------------+\n",
        )?;
        for region in self.iter() {
            f.write_fmt(format_args!(
                "| {:?}\t | {:#016x} | {:#016x} | {:>16} |\n",
                region.kind,
                region.start,
                region.end,
                HumanBytes::from(region.end - region.start)
            ))?;
        }
        f.write_str("+--------------+------------------+------------------+-------------------+\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_enum_has_precedence() {
        assert!(PhysMemoryKind::None < PhysMemoryKind::Free);
        assert!(PhysMemoryKind::Free < PhysMemoryKind::Reserved);
    }

    #[test]
    fn test_insert_one_element() {
        let mut mm = PhysMemoryMap::<3>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(mm.len, 1);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_insert_two_element() {
        let mut mm = PhysMemoryMap::<3>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 1
            }),
            Ok(())
        );

        assert_eq!(mm.len, 2);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 1
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_insert_all_elements() {
        let mut mm = PhysMemoryMap::<3>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(2, PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            }),
            Ok(())
        );

        assert_eq!(mm.len, 3);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            }
        ]);
    }

    #[test]
    fn test_insert_middle_elements() {
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            }),
            Ok(())
        );

        assert_eq!(mm.len, 3);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_remove_last_element() {
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(2, PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            }),
            Ok(())
        );

        assert_eq!(mm.remove_raw(2), Ok(()));

        assert_eq!(mm.len, 2);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_remove_middle_element() {
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(2, PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            }),
            Ok(())
        );

        assert_eq!(mm.remove_raw(1), Ok(()));

        assert_eq!(mm.len, 2);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_remove_first_element() {
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.insert_raw(0, PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(1, PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            }),
            Ok(())
        );

        assert_eq!(
            mm.insert_raw(2, PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            }),
            Ok(())
        );

        assert_eq!(mm.remove_raw(0), Ok(()));

        assert_eq!(mm.len, 2);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 2
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_add_one_region() {
        lldebug::testing_stdout!();
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.add_region(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 0,
                end: 10
            }),
            Ok(())
        );

        assert_eq!(mm.len, 2, "Array len didnt match! {:#?}", mm.borders);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 10
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_add_two_region_no_overlap() {
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.add_region(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 0,
                end: 10
            }),
            Ok(()),
            "{:#?}",
            mm.borders
        );

        assert_eq!(
            mm.add_region(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 20,
                end: 30
            }),
            Ok(()),
            "{:#?}",
            mm.borders
        );

        assert_eq!(mm.len, 4, "Array len didnt match! {:#?}", mm.borders);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 10
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 20
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 30
            }
        ]);
    }

    #[test]
    fn test_add_two_region_middle_overlap_change_last() {
        let mut mm = PhysMemoryMap::<4>::new();

        assert_eq!(
            mm.add_region(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 0,
                end: 10
            }),
            Ok(()),
            "{:#?}",
            mm.borders
        );

        assert_eq!(
            mm.add_region(PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 5,
                end: 20
            }),
            Ok(()),
            "{:#?}",
            mm.borders
        );

        assert_eq!(mm.len, 3, "Array len didnt match! {:#?}", mm.borders);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 5
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 20
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0
            }
        ]);
    }

    #[test]
    fn test_complex_overlap_region() {
        lldebug::testing_stdout!();
        let mut mm = PhysMemoryMap::<10>::new();

        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Free,
            start: 5,
            end: 14,
        })
        .unwrap();
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Reserved,
            start: 0,
            end: 6,
        })
        .unwrap();
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Special,
            start: 3,
            end: 10,
        })
        .unwrap();
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Bootloader,
            start: 14,
            end: 18,
        })
        .unwrap();
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Free,
            start: 14,
            end: 18,
        })
        .unwrap();

        assert_eq!(mm.len, 5, "Array len didnt match! {:#?}", mm.borders);
        assert_eq!(
            mm.borders,
            [
                PhysMemoryBorder {
                    kind: PhysMemoryKind::Reserved,
                    address: 0
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::Special,
                    address: 3
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::Free,
                    address: 10
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::Bootloader,
                    address: 14
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::None,
                    address: 18
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::None,
                    address: 0
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::None,
                    address: 0
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::None,
                    address: 0
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::None,
                    address: 0
                },
                PhysMemoryBorder {
                    kind: PhysMemoryKind::None,
                    address: 0
                },
            ],
            "{:#?}",
            mm.borders
        );
    }

    #[test]
    fn test_real_world() {
        let real_mem_map = [
            PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 0,
                end: 654336,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 654336,
                end: 654336 + 1024,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 983040,
                end: 983040 + 65536,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 1048576,
                end: 1048576 + 267255808,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 268304384,
                end: 268304384 + 131072,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 4294705152,
                end: 4294705152 + 262144,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 1086626725888,
                end: 1086626725888 + 12884901888,
            },
        ];

        let mut mm = PhysMemoryMap::<20>::new();

        for entry in real_mem_map.iter() {
            mm.add_region(entry.clone()).unwrap();
        }

        assert_eq!(mm.len, 11);
    }

    #[test]
    fn test_iter() {
        let mut mm = PhysMemoryMap::<10>::new();

        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Free,
            start: 0,
            end: 10,
        })
        .unwrap();

        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Reserved,
            start: 5,
            end: 20,
        })
        .unwrap();

        let mut iter = mm.iter();

        assert_eq!(
            iter.next(),
            Some(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 0,
                end: 5
            })
        );
        assert_eq!(
            iter.next(),
            Some(PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 5,
                end: 20
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_find_cont_of() {
        let mut mm = PhysMemoryMap::<10>::new();

        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Free,
            start: 1,
            end: 64,
        })
        .unwrap();

        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Reserved,
            start: 32,
            end: 64,
        })
        .unwrap();

        assert_eq!(
            mm.find_continuous_of(PhysMemoryKind::Free, 16, 8, 0),
            Some(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 8,
                end: 24
            })
        );
        assert_eq!(
            mm.find_continuous_of(PhysMemoryKind::Free, 8, 8, 10),
            Some(PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 16,
                end: 24
            })
        );
    }

    #[test]
    fn test_real_add_ss_regions_to_mm() {
        let mut mm = PhysMemoryMap::<16>::new();

        const EXAMPLE_MM: [PhysMemoryEntry; 7] = [
            PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 0,
                end: 654336,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 654336,
                end: 655360,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 983040,
                end: 1048576,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: 1048576,
                end: 268304384,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 268304384,
                end: 268435456,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 4294705152,
                end: 4294967296,
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: 1086626725888,
                end: 1099511627776,
            },
        ];

        for example in EXAMPLE_MM.iter() {
            mm.add_region(example).unwrap();
        }

        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Bootloader,
            start: 2097152,
            end: 2124304,
        })
        .unwrap();
        mm.add_region(PhysMemoryEntry {
            kind: PhysMemoryKind::Bootloader,
            start: 4194304,
            end: 4218432,
        })
        .unwrap();

        assert_eq!(mm.len, 15, "{:#?}", mm.borders);
        assert_eq!(mm.borders, [
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 654336,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 655360,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 983040,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 1048576,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Bootloader,
                address: 2097152,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 2124304,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Bootloader,
                address: 0x400000,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Free,
                address: 0x405e40,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 268304384,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 268435456,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 4294705152,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 4294967296,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::Reserved,
                address: 1086626725888,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 1099511627776,
            },
            PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0,
            },
        ],);
    }
}
