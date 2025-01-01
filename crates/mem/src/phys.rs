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
            "Trying to insert segment(kind={:?}, start={}, end={}) into the following array:\n  {:?}",
            kind,
            start,
            end,
            &self.borders[..self.len]
        );
        let mut end_segment_kind = PhysMemoryKind::None;
        let mut start_i = 0;

        // 1. We need iter until we find a border with a higher address,
        //    or until we find reach a border with a higher address then our
        //    end address.
        //
        //    We also need to keep going if the segment is of a higher type.
        while start_i <= self.len
            && self
                .borders
                .get(start_i)
                .is_some_and(|bor| bor.address < start || bor.kind > kind)
        {
            // 1.1. If this segment contains a larger address than our end, we
            //      cannot place our new segment into the array.
            if self
                .borders
                .get(start_i)
                .is_some_and(|bor| bor.address >= end)
            {
                logln!("Pushed start further then end, cannot insert segment!");
                return Ok(());
            }

            // 1.2. We need to inc our index.
            start_i += 1;
        }

        // 2. If we pushed our start segment up, we don't need to insert.
        if start_i > 0
            && self
                .borders
                .get(start_i)
                .is_some_and(|bor| bor.address >= start)
        {
            logln!("Segment was pushed up due to lower precedence.");
            // 2.1. If our end segment is lower then the next border
            //      we change the type of our end border to the type
            //      of the old segment.
            if self.len > start_i + 1 && self.borders[start_i + 1].address > end {
                logln!("No border changes until end segment, saving old segment kind...");
                end_segment_kind = self.borders[start_i].kind;
            }

            logln!(
                "Changing border (at idx={}) to kind={:?} (old_kind={:?})",
                start_i,
                kind,
                self.borders[start_i]
            );

            // 2.2. We change the old segment to our segment kind.
            self.borders[start_i].kind = kind;
        }
        // 3. If we didn't push our start segment up, we can insert one now.
        else {
            logln!("Didn't push start border up, inserting new!");
            self.insert_raw(start_i, PhysMemoryBorder {
                kind,
                address: start,
            })?;
        }

        let mut end_i = self.len;

        // 3. We need to iter until we find a border that is of a lower or
        //    equal type to our segment kind.
        //
        //    We also need to find where the end segment should go based
        //    on address.
        while end_i > start_i
            && self
                .borders
                .get(end_i)
                .is_some_and(|bor| bor.kind < kind || bor.address > end)
        {
            end_i -= 1;
        }

        // 4. If we didn't push down our end segment, we need to insert one.
        if !self.borders.get(end_i).is_some_and(|bor| bor.address < end) {
            logln!(
                "Didn't push down end segment, so inserting a new one (idx={}, kind={:?}, address={})",
                end_i,
                end_segment_kind,
                end
            );
            self.insert_raw(end_i, PhysMemoryBorder {
                kind: end_segment_kind,
                address: end,
            })?;
        }

        let mut inner = start_i + 1;
        let mut have_hit_higher = false;

        // 5. We need to remove all segments inbetween that is of a lower
        //    type.
        //
        //    If we find a higher border type, the next border (of a lower type)
        //    needs to be changed to our type.
        //
        while inner < end_i {
            if self.borders.get(inner).is_some_and(|bor| bor.kind > kind) {
                logln!("Have hit a border with a higher type! (idx={})", inner);
                have_hit_higher = true;
                inner += 1;
                continue;
            }

            if have_hit_higher {
                logln!("Changing border (at idx={}) to kind={:?}", inner, kind);
                self.borders[inner].kind = kind;
                inner += 1;
                continue;
            }

            self.remove_raw(inner)?;
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

        if index + 1 > self.len {
            return Err(crate::MemoryError::InvalidSize);
        }

        // if the index is at the end of the array, we don't need to move any
        // of the elements.
        if index + 1 == self.len {
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
        if self.len == 0 {
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
        self.borders.copy_within(index..border_len, index - 1);
        self.len -= 1;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
