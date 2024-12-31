/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

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
enum MemoryRegionCmp {
    /// `rhs` fully outside `self`
    ///
    /// ```not_rust
    ///                [=======self========]
    /// [====rhs====]
    /// ```
    None,
    /// `rhs` ends inside `self`
    ///
    /// ```not_rust
    ///     [=======self========]
    /// [====rhs====]
    /// ```
    Start,
    /// `rhs` is entirely contained within `self`
    ///
    /// ```not_rust
    ///     [=======self========]
    ///         [====rhs====]
    /// ```
    Inside,
    /// `rhs` starts inside `self`
    ///
    /// ```not_rust
    ///     [=======self========]
    ///                   [====rhs====]
    /// ```
    End,
    /// `rhs` ends outside of `self`
    ///
    /// ```not_rust
    ///     [=======self========]
    ///  [==========rhs============]
    /// ```
    Full,
    /// `rhs` is exactly `self`
    ///
    /// ```not_rust
    ///     [=======self========]
    ///     [=======rhs=========]
    /// ```
    Exact,
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

    /// Combines `self` with `rhs`, extending `self` to contain `rhs`
    pub const fn melt_right(self, rhs: Self) -> Self {
        Self {
            kind: self.kind,
            start: self.start,
            end: rhs.end,
        }
    }

    /// Combines `rhs` with `self`, extending `rhs` to contain `self`
    pub const fn melt_left(self, rhs: Self) -> Self {
        Self {
            kind: self.kind,
            start: rhs.start,
            end: self.end,
        }
    }

    pub fn melt_both(self, rhs: Self) -> Self {
        Self {
            kind: self.kind,
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }

    const fn range_cmp(&self, rhs: &Self) -> MemoryRegionCmp {
        match () {
            // L:        [===]
            // R: [===]
            _ if self.start > rhs.start && self.start > rhs.end => MemoryRegionCmp::None,
            // L:      [========]
            // R: [========]
            _ if self.start > rhs.start && self.start < rhs.end && self.end > rhs.end => {
                MemoryRegionCmp::Start
            }
            // L:      [========]
            // R:        [====]
            _ if self.start < rhs.end && self.end > rhs.end => MemoryRegionCmp::Inside,
            // L:      [========]
            // R:        [========]
            _ if self.start < rhs.start && self.end > rhs.start && self.end < rhs.end => {
                MemoryRegionCmp::End
            }
            // L:      [========]
            // R:    [============]
            _ if self.start > rhs.start && self.end < rhs.end => MemoryRegionCmp::Full,
            // L:      [========]
            // R:      [========]
            _ => MemoryRegionCmp::Exact,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhysMemoryMap<const N: usize> {
    regions: [PhysMemoryEntry; N],
}

impl<const N: usize> PhysMemoryMap<N> {
    pub fn new(old_map: &[impl MemoryDesc]) -> Self {
        let mut nm = Self {
            regions: [PhysMemoryEntry::empty(); N],
        };

        nm.regions
            .iter_mut()
            .zip(old_map.iter().filter(|old| {
                old.memory_kind() != PhysMemoryKind::None && old.memory_start() != old.memory_end()
            }))
            .for_each(|(new, old)| {
                new.kind = old.memory_kind();
                new.start = old.memory_start();
                new.end = old.memory_end();
            });

        nm.regions.sort_unstable_by(|l, r| l.start.cmp(&r.start));

        nm
    }

    pub fn add_region(&mut self, region: PhysMemoryEntry) -> Result<(), crate::MemoryError> {
        todo!()
    }
}
