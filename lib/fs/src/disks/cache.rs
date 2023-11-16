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

use qk_alloc::vec::{ToVec, Vec};

use crate::FsResult;

/// # Cache State
/// The state of the bytes just inserted into the cache. Informs the cache if it can delete this
/// entry or if it needs to be flushed first.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CacheState {
    /// # Requires Flush
    /// This cache insertion requires the memory to be flushed to disk before being deleted.
    RequiresFlush,
    /// # Disk Backed
    /// This cache insertion does not require a flush to be deleted, and can be deleted at anytime.
    DiskBacked,
}

/// # Cache Entry
/// One entry in the disk cache. Used to store infomation about the data, along with the data
/// itself.
#[derive(Debug)]
struct CacheEntry {
    age: usize,
    sector: usize,
    data: Vec<u8>,
    state: CacheState,
}

/// # Disk Cache
/// Cached disk sectors or infomation that is otherwise expensive to read. Used mainly to store
/// sectors for reading and writing. Retains state about the allocations, and will automaticlly
/// prune ones that are too old when buffer size is reached.
#[derive(Debug)]
pub struct DiskCache {
    expected_max_chunks: usize,
    cache: Vec<CacheEntry>,
}

impl DiskCache {
    /// # New
    /// Creates a new DiskCache with the provided infomation.
    pub fn new(max_chunks: usize) -> Self {
        Self {
            expected_max_chunks: max_chunks,
            cache: Vec::new(),
        }
    }

    /// # Pruge Unneeded
    /// Removes allocations who are too old. If the age is larger then the expected_max_chunks,
    /// then the allocation gets removed.
    fn purge_unneeded(&mut self) {
        if !self.cache_quota_reached() {
            return;
        }

        self.cache.retain(|entry| {
            entry.state == CacheState::RequiresFlush || entry.age < self.expected_max_chunks
        })
    }

    /// # Cache Quota Reached
    /// Checks if the quota of disks is larger then the expected max
    fn cache_quota_reached(&self) -> bool {
        self.cache.len() >= self.expected_max_chunks
    }

    /// # Increment Buffer Age
    /// Increments the age of each of the elements in the buffer.
    fn increment_buffer_age(&mut self) {
        self.cache.iter_mut().for_each(|entry| entry.age += 1)
    }

    /// # Len
    /// Gets how many items are in the cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// # Insert
    /// Adds an allocation into the cache.
    pub fn insert(&mut self, state: CacheState, sector: usize, data: &[u8]) {
        self.purge_unneeded();
        if state == CacheState::DiskBacked && self.cache_quota_reached() {
            return;
        }

        self.increment_buffer_age();
        self.invalidate(sector);
        self.cache.push(CacheEntry {
            age: 0,
            sector,
            data: ToVec::to_vec(data), /*rust_analyzer thinks we are talking about the alloc vector if we just use .to_vec()*/
            state,
        });
    }

    /// # Invalidate
    /// Removes an entry from the cache if its found. If the entry is not found, it simply does
    /// nothing.
    pub fn invalidate(&mut self, sector: usize) {
        self.cache.retain(|entry| entry.sector != sector)
    }

    /// # Invalidate All
    /// Removes all elements from the cache.
    pub fn invalidate_all(&mut self, sector: usize) {
        self.cache.remove_all();
    }

    /// # Get Entry
    /// Gets the data stored in an entry.
    pub fn get_entry<'a>(&'a self, sector: usize) -> Option<&'a [u8]> {
        self.cache.iter().find_map(|entry| {
            if entry.sector == sector {
                Some(entry.data.as_slice())
            } else {
                None
            }
        })
    }

    /// # Flush Required
    /// Gets a refrence to the bytes for all of the flushed required entries in the cache. Then
    /// prunes the cache to remove old allocations.
    pub fn flush_required<Function>(&mut self, mut f: Function) -> FsResult<()>
    where
        Function: FnMut(usize, &[u8]) -> FsResult<()>,
    {
        self.cache
            .iter_mut()
            .try_for_each(|entry| -> FsResult<()> {
                if entry.state == CacheState::DiskBacked {
                    return Ok(());
                }

                f(entry.sector, entry.data.as_slice())?;
                entry.state = CacheState::DiskBacked;

                Ok(())
            })?;

        self.purge_unneeded();

        Ok(())
    }
}
