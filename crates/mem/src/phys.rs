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

use lldebug::logln;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhysMemoryKind {
    None,
    Free,
    Reserved,
    Special,
    Bootloader,
    Kernel,
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
    kind: PhysMemoryKind,
    start: u64,
    end: u64,
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
}

#[derive(Clone, Copy, Debug)]
pub struct PhysMemoryMap<const N: usize> {
    borders: [PhysMemoryBorder; N],
    len: usize,
}

impl<const N: usize> PhysMemoryMap<N> {
    pub fn new() -> Self {
        Self {
            borders: [PhysMemoryBorder {
                kind: PhysMemoryKind::None,
                address: 0,
            }; N],
            len: 0,
        }
    }

    fn find_start_index(&self, kind: PhysMemoryKind, start: u64, end: u64) -> Option<usize> {
        let mut ret = None;

        for (i, bor) in self
            .borders
            .iter()
            .enumerate()
            .take_while(|(_, bor)| bor.address < end)
            .filter(|(_, bor)| bor.kind <= kind)
            .take_while(|(i, _)| *i <= self.len)
        {
            if bor.address == start && bor.kind <= kind {
                return Some(i);
            }

            if bor.address < start && self.borders.get(i + 1).is_some_and(|a| a.address >= end) {
                return Some(i + 1);
            }

            ret = Some(i);
        }

        ret
    }

    pub fn add_region(&mut self, region: impl MemoryDesc) -> Result<(), crate::MemoryError> {
        let kind = region.memory_kind();
        let start = region.memory_start();
        let end = region.memory_end();

        if start >= end {
            return Err(crate::MemoryError::InvalidSize);
        }

        // If its a None, we don't need to do any work
        if kind == PhysMemoryKind::None {
            return Ok(());
        }

        // Our structure looks like this:
        // ```no_rust
        //IDX:    0          1     2     3
        //TYP:    F          U     F     N
        //        |          |     |     |
        //  F:    +----------+     +-----+
        //  U:          F    +-----+  F
        //                      U
        // ```

        logln!(
            "ADD (kind={:?}, start={}, end={}) -- {:#?}",
            kind,
            start,
            end,
            &self.borders[..self.len]
        );

        let Some(start_i) = self.find_start_index(kind, start, end) else {
            // Region is being overriden by others with higher precedence.
            return Ok(());
        };

        let want_insert_start = PhysMemoryBorder {
            kind,
            address: self.borders[..self.len]
                .get(start_i)
                .map(|b| b.address.max(start))
                .unwrap_or(start),
        };

        let mut larger_region = false;
        let mut i = start_i;

        while i < self.len {
            if self.borders[i].kind > kind {
                larger_region = true;
                i += 1;
                continue;
            }

            if larger_region {
                larger_region = false;
                self.borders[i].kind = kind;
                i += 1;
                continue;
            }

            logln!("REMOVE i={} bor={:?}", i, self.borders[i]);
            self.remove_raw(i)?;
        }

        logln!("AFTER {:#?}", self.borders);

        self.insert_raw(start_i, want_insert_start)?;

        if let Some((end_index, old_end)) = self
            .borders
            .iter()
            .enumerate()
            .skip(start_i + 1)
            .take(self.len)
            .take_while(|(_, bor)| bor.address <= end)
            .filter(|(_, bor)| bor.kind < kind)
            .last()
        {
            self.insert_raw(end_index, PhysMemoryBorder {
                kind: old_end.kind,
                address: self.borders[..self.len]
                    .get(end_index)
                    .map(|b| b.address.min(end))
                    .unwrap_or(end),
            })?;
        }

        Ok(())
    }

    fn insert_raw(
        &mut self,
        index: usize,
        border: PhysMemoryBorder,
    ) -> Result<(), crate::MemoryError> {
        logln!("INSERT: idx={index} bor={border:?}");
        if self.len == self.borders.len() {
            return Err(crate::MemoryError::ArrayTooSmall);
        }

        if index > self.len {
            logln!("1: invalid -- {index} <- {border:?}");
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
            logln!("2: invalid");
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
        lldebug::testing_stdout!();
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
    fn test_find_s_index() {
        let mut mm = PhysMemoryMap::<10>::new();

        assert_eq!(mm.find_start_index(PhysMemoryKind::Free, 0, 10), Some(0));
        assert_eq!(
            mm.find_start_index(PhysMemoryKind::Reserved, 0, 10),
            Some(0)
        );

        // 0    1                   2          3              4
        // X    X                   $+1
        // 2    3                   0          2              0
        // |    |                   |          |              |
        // +----+-------------------+          +--------------+
        //    2           3                            2
        //                            ^^^^^^^^^^
        //                                1
        mm.len = 5;
        mm.borders[0] = PhysMemoryBorder {
            kind: PhysMemoryKind::Reserved,
            address: 0,
        };
        mm.borders[1] = PhysMemoryBorder {
            kind: PhysMemoryKind::Special,
            address: 3,
        };
        mm.borders[2] = PhysMemoryBorder {
            kind: PhysMemoryKind::None,
            address: 10,
        };
        mm.borders[3] = PhysMemoryBorder {
            kind: PhysMemoryKind::Reserved,
            address: 20,
        };
        mm.borders[4] = PhysMemoryBorder {
            kind: PhysMemoryKind::None,
            address: 30,
        };

        assert_eq!(mm.find_start_index(PhysMemoryKind::Free, 5, 14), Some(2));
        assert_eq!(
            mm.find_start_index(PhysMemoryKind::Bootloader, 3, 14),
            Some(1)
        );
        assert_eq!(mm.find_start_index(PhysMemoryKind::Free, 0, 10), None);
        assert_eq!(mm.find_start_index(PhysMemoryKind::Special, 3, 10), Some(1));
        assert_eq!(mm.find_start_index(PhysMemoryKind::Free, 11, 20), Some(3));
        assert_eq!(mm.find_start_index(PhysMemoryKind::Kernel, 11, 20), Some(3));
        assert_eq!(mm.find_start_index(PhysMemoryKind::Kernel, 20, 25), Some(3));

        mm.len = 2;
        mm.borders[0] = PhysMemoryBorder {
            kind: PhysMemoryKind::Free,
            address: 0,
        };
        mm.borders[1] = PhysMemoryBorder {
            kind: PhysMemoryKind::None,
            address: 10,
        };

        assert_eq!(
            mm.find_start_index(PhysMemoryKind::Reserved, 5, 14),
            Some(1)
        );
    }
}
