/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
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

use crate::locks::{LockEncouragement, RwCriticalLock, RwYieldLock};
use alloc::{
    collections::btree_map::BTreeMap,
    string::String,
    sync::{Arc, Weak},
};
use boolvec::BoolVec;
use elf::{elf_owned::ElfOwned, tables::SegmentKind};
use mem::{
    addr::VirtAddr,
    paging::VmPermissions,
    vm::{InsertVmObjectError, VmFillAction, VmProcess, VmRegion},
};
use scheduler::Scheduler;
use thread::{ThreadId, WeakThread};
use util::consts::PAGE_4K;
use vm_elf::VmElfInject;

pub mod scheduler;
pub mod task;
pub mod thread;
mod vm_elf;

pub type ProcessEntry = VirtAddr;
pub type ProcessId = usize;
pub type RefProcess = Arc<Process>;
pub type WeakProcess = Weak<Process>;

/// A complete execution unit, memory map, threads, etc...
#[derive(Debug)]
pub struct Process {
    /// The unique id of this process
    id: ProcessId,
    /// The name of this process
    name: String,
    /// Weak references to all threads within this process.
    ///
    /// Threads carry strong references to their process, and are the actual scheduling artifacts.
    threads: RwYieldLock<BTreeMap<ThreadId, WeakThread>>,
    /// Thread allocation bitmap
    thread_id_alloc: RwYieldLock<BoolVec>,
    /// The memory map of this process
    // FIXME: Need to convert `VmProcess` to not use locks
    vm: RwCriticalLock<VmProcess>,
}

impl Process {
    /// Create a new process
    pub fn new(name: String) -> RefProcess {
        let s = Scheduler::get();

        let proc = Arc::new(Self {
            id: s.alloc_pid(),
            name,
            threads: RwYieldLock::new(BTreeMap::new()),
            thread_id_alloc: RwYieldLock::new(BoolVec::new()),
            vm: RwCriticalLock::new(s.fork_kernel_vm()),
        });
        s.register_new_process(proc.clone());

        proc
    }

    /// Add an ELF mapping to this process's memory map
    pub fn map_elf(&self, elf: Arc<ElfOwned>) -> ProcessEntry {
        let vm_lock = self.vm.write();
        let elf_fill = VmElfInject::new(elf.clone()).fill_action();

        elf.elf()
            .program_headers()
            .unwrap()
            .iter()
            .filter(|header| header.segment_kind() == SegmentKind::Load)
            .for_each(|header| {
                let header_perms = VmPermissions::none()
                    .set_exec_flag(header.is_executable())
                    .set_read_flag(true)
                    .set_write_flag(header.is_writable());

                let header_region =
                    VmRegion::from_kbh((header.expected_vaddr(), header.in_mem_size()));

                match vm_lock.inplace_new_vmobject(
                    header_region,
                    header_perms,
                    elf_fill.clone(),
                    false,
                ) {
                    Ok(_) => (),
                    Err(InsertVmObjectError::Overlapping {
                        existing: _,
                        attempted: _,
                    }) => (),
                    Err(err) => panic!("{:#?}", err),
                }
            });

        elf.elf().entry_point().unwrap().into()
    }

    /// Add a new anonymous memory mapping
    pub fn map_anon(&self, region: VmRegion, perm: VmPermissions) {
        let vm_lock = self.vm.write();

        vm_lock
            .inplace_new_vmobject(region, perm, VmFillAction::Scrub(0), false)
            .unwrap();
    }

    /// Allocate a new thread id
    pub fn alloc_thread_id(&self) -> ThreadId {
        // Moderate lock because holding this lock means we cannot spawn any new threads for this process, but
        // we can still execute the current threads.
        let mut thread_ids = self.thread_id_alloc.write(LockEncouragement::Moderate);

        let new_thread_id = thread_ids
            .find_first_of(false)
            .expect("Cannot allocate a new thread id");
        thread_ids.set(new_thread_id, true);

        new_thread_id
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let s = Scheduler::get();
        s.remove_process(self);
    }
}
