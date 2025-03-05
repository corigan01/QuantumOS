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

use core::sync::atomic::AtomicBool;

use crate::locks::{LockEncouragement, RwCriticalLock, RwYieldLock};
use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use boolvec::BoolVec;
use elf::elf_owned::ElfOwned;
use lldebug::{logln, warnln};
use mem::{
    addr::VirtAddr,
    paging::VmPermissions,
    vm::{VmFillAction, VmProcess, VmRegion},
};
use quantum_portal::{HandleUpdateKind, WaitSignal};
use scheduler::Scheduler;
use thread::{ThreadId, WeakThread};
use vm_elf::VmElfInject;

pub mod scheduler;
pub mod task;
pub mod thread;
mod vm_elf;

pub type ProcessEntry = VirtAddr;
pub type ProcessId = usize;
pub type RefProcess = Arc<Process>;
pub type WeakProcess = Weak<Process>;

#[derive(Debug)]
pub enum ProcessHandle {
    /// A socket that can accept connections
    ConnectionHandle {
        connections: Vec<u64>,
        host_name: String,
    },
    /// A handle socket owner
    HostTwoWay {
        /// Bytes that the host can read
        host_rx: RwYieldLock<VecDeque<u8>>,
        /// Bytes the host sent
        host_tx: RwYieldLock<VecDeque<u8>>,
        /// Ref to the client
        client: WeakProcess,
        /// Id on the client side
        id: u64,
    },
    /// A handle socket client
    ClientTwoWay {
        /// Ref to the host
        host: WeakProcess,
        /// Id on the host
        id: u64,
    },
    Disconnected,
}

#[derive(Debug)]
pub struct ProcessHandleManager {
    id_alloc: BoolVec,
    handles: BTreeMap<u64, ProcessHandle>,
}

impl ProcessHandleManager {
    pub fn new() -> Self {
        Self {
            id_alloc: BoolVec::new(),
            handles: BTreeMap::new(),
        }
    }

    /// Create a new handle id
    fn alloc_handle_id(&mut self) -> u64 {
        let id = self.id_alloc.find_first_of(false).unwrap_or(0);
        self.id_alloc.set(id, true);

        id as u64
    }

    /// Destroy this handle id
    fn dealloc_handle_id(&mut self, handle: u64) {
        assert!(
            self.id_alloc.get(handle as usize),
            "Tried to free an invalid handle!"
        );
        self.id_alloc.set(handle as usize, false);
    }

    /// Disconnect handle
    pub fn disconnect_handle(&mut self, handle: u64) -> ProcessHandle {
        self.dealloc_handle_id(handle);
        self.handles
            .insert(handle, ProcessHandle::Disconnected)
            .unwrap()
    }

    /// Create a new connection handle
    pub fn new_connection_handle(&mut self, host_name: String) -> u64 {
        let id = self.alloc_handle_id();
        self.handles.insert(
            id,
            ProcessHandle::ConnectionHandle {
                host_name,
                connections: Vec::new(),
            },
        );

        id
    }

    /// Create a new host and client handle pair
    fn new_handle_pair(owner: RefProcess, host_id: u64, client: RefProcess) -> (u64, u64) {
        let mut owner_process = owner.handles.write(LockEncouragement::Strong);
        let mut client_process = client.handles.write(LockEncouragement::Strong);

        let owner_id = owner_process.alloc_handle_id();
        let client_id = client_process.alloc_handle_id();

        match owner_process.handles.get_mut(&host_id) {
            Some(ProcessHandle::ConnectionHandle {
                connections,
                host_name: _,
            }) => {
                connections.push(owner_id);
            }
            _ => unreachable!(),
        }

        owner_process.handles.insert(
            owner_id,
            ProcessHandle::HostTwoWay {
                host_rx: RwYieldLock::new(VecDeque::new()),
                host_tx: RwYieldLock::new(VecDeque::new()),
                client: Arc::downgrade(&client),
                id: client_id,
            },
        );
        client_process.handles.insert(
            client_id,
            ProcessHandle::ClientTwoWay {
                host: Arc::downgrade(&owner),
                id: owner_id,
            },
        );

        owner
            .signals
            .write(LockEncouragement::Moderate)
            .push_back(WaitSignal::HandleUpdate {
                handle: host_id,
                kind: HandleUpdateKind::NewConnection {
                    new_handle: owner_id,
                },
            });

        (owner_id, client_id)
    }
}

/// A complete execution unit, memory map, threads, etc...
#[derive(Debug)]
pub struct Process {
    /// The unique id of this process
    pub id: ProcessId,
    /// The name of this process
    pub name: String,
    /// Weak references to all threads within this process.
    ///
    /// Threads carry strong references to their process, and are the actual scheduling artifacts.
    threads: RwYieldLock<BTreeMap<ThreadId, WeakThread>>,
    /// Thread allocation bitmap
    thread_id_alloc: RwYieldLock<BoolVec>,
    /// Handle ID alloc
    handles: RwYieldLock<ProcessHandleManager>,
    /// The memory map of this process
    // FIXME: Need to convert `VmProcess` to not use locks
    vm: RwCriticalLock<VmProcess>,
    /// Has this process been killed, or exited?
    pub dead: AtomicBool,
    /// Signals for userspace
    signals: RwYieldLock<VecDeque<WaitSignal>>,
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
            handles: RwYieldLock::new(ProcessHandleManager::new()),
            dead: AtomicBool::new(false),
            signals: RwYieldLock::new(VecDeque::new()),
        });
        s.register_new_process(proc.clone());

        proc
    }

    /// Add an ELF mapping to this process's memory map
    pub fn map_elf(&self, elf: Arc<ElfOwned>) -> ProcessEntry {
        let mut vm_lock = self.vm.write();
        let elf_fill = VmElfInject::new(elf.clone()).fill_action();

        let header_perms = VmPermissions::none()
            .set_user_flag(true)
            .set_exec_flag(true)
            .set_read_flag(true)
            .set_write_flag(true);

        let (start_addr, end_addr) = elf.elf().vaddr_range().unwrap();
        vm_lock
            .inplace_new_vmobject(
                VmRegion::from_containing(start_addr.into(), end_addr.into()),
                header_perms,
                elf_fill.clone(),
                false,
            )
            .unwrap();

        elf.elf().entry_point().unwrap().into()
    }

    /// Add a new anonymous memory mapping
    pub fn map_anon(&self, region: VmRegion, perm: VmPermissions) {
        let mut vm_lock = self.vm.write();

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

    pub fn disconnect_handle(host: RefProcess, handle: u64) {
        // If this handle doesn't exist, skip
        if !host
            .handles
            .read(LockEncouragement::Weak)
            .id_alloc
            .get(handle as usize)
        {
            warnln!("Tried to disconnect an invalid handle (id={handle})");
            return;
        }

        host.signals
            .write(LockEncouragement::Moderate)
            .push_back(WaitSignal::HandleUpdate {
                handle,
                kind: HandleUpdateKind::Disconnected,
            });

        match host
            .handles
            .write(LockEncouragement::Moderate)
            .disconnect_handle(handle)
        {
            ProcessHandle::ConnectionHandle {
                connections,
                host_name: _,
            } => {
                for self_connection in connections {
                    Self::disconnect_handle(host.clone(), self_connection);
                }
            }
            ProcessHandle::HostTwoWay {
                host_rx: _,
                host_tx: _,
                client,
                id,
            } => {
                if let Some(client_upgrade) = client.upgrade() {
                    // Disconnect client's socket
                    Self::disconnect_handle(client_upgrade, id);
                }
            }
            ProcessHandle::ClientTwoWay { .. } => (),
            ProcessHandle::Disconnected => (),
        }
    }

    /// Create a new connection handle
    pub fn new_connection_handle(host: RefProcess, name: String) -> Option<u64> {
        let s = Scheduler::get();
        if s.serve_sockets.lock().get(&name).is_some() {
            return None;
        }

        let handle_id = host
            .handles
            .write(LockEncouragement::Moderate)
            .new_connection_handle(name.clone());

        s.serve_sockets
            .lock()
            .insert(name, (Arc::downgrade(&host), handle_id));
        Some(handle_id)
    }

    /// Create a new connection handle
    pub fn new_handle_pair(host: RefProcess, host_id: u64, client: RefProcess) -> (u64, u64) {
        ProcessHandleManager::new_handle_pair(host, host_id, client)
    }

    /// Put data into this handle's rx
    fn remote_tx(&self, id: u64, data: &[u8]) -> Result<usize, HandleError> {
        let handle_lock = self.handles.read(LockEncouragement::Weak);

        let Some(handle_info) = handle_lock.handles.get(&id) else {
            return Err(HandleError::HandleDoesntExist(id));
        };

        match handle_info {
            ProcessHandle::HostTwoWay {
                host_rx,
                host_tx: _,
                client: _,
                id: _,
            } => {
                let mut rx_lock = host_rx.write(LockEncouragement::Moderate);
                rx_lock.extend(data.iter());
                Ok(data.len())
            }
            _ => Err(HandleError::InvalidSocketKind),
        }
    }

    /// Send data over this socket
    pub fn handle_tx(&self, id: u64, data: &[u8]) -> Result<usize, HandleError> {
        let handle_lock = self.handles.read(LockEncouragement::Weak);

        let Some(handle_info) = handle_lock.handles.get(&id) else {
            return Err(HandleError::HandleDoesntExist(id));
        };

        match handle_info {
            ProcessHandle::HostTwoWay {
                host_rx: _,
                host_tx,
                client: _,
                id: _,
            } => {
                let mut tx_lock = host_tx.write(LockEncouragement::Moderate);
                tx_lock.extend(data.iter());
                Ok(data.len())
            }
            ProcessHandle::ClientTwoWay { host, id } => {
                let host = host.upgrade().ok_or(HandleError::HostDisconnect)?;
                host.signals.write(LockEncouragement::Moderate).push_back(
                    WaitSignal::HandleUpdate {
                        handle: *id,
                        kind: HandleUpdateKind::ReadReady,
                    },
                );
                host.remote_tx(*id, data)
            }
            _ => Err(HandleError::InvalidSocketKind),
        }
    }

    /// Recv data over this socket
    fn remote_rx(&self, id: u64, data: &mut [u8]) -> Result<usize, HandleError> {
        let handle_lock = self.handles.read(LockEncouragement::Weak);

        let Some(handle_info) = handle_lock.handles.get(&id) else {
            return Err(HandleError::HandleDoesntExist(id));
        };

        match handle_info {
            ProcessHandle::HostTwoWay {
                host_rx: _,
                host_tx,
                client: _,
                id: _,
            } => {
                let mut tx_lock = host_tx.write(LockEncouragement::Moderate);
                if tx_lock.len() == 0 {
                    self.signals.write(LockEncouragement::Moderate).push_back(
                        WaitSignal::HandleUpdate {
                            handle: id,
                            kind: HandleUpdateKind::WriteReady,
                        },
                    );
                    return Err(HandleError::WouldBlock);
                }

                let mut bytes = 0;
                for entry_mut in data.iter_mut() {
                    let Some(front) = tx_lock.pop_front() else {
                        return Ok(bytes);
                    };

                    bytes += 1;
                    *entry_mut = front;
                }

                Ok(bytes)
            }
            _ => Err(HandleError::InvalidSocketKind),
        }
    }

    /// Recv data from this socket
    pub fn handle_rx(&self, id: u64, data: &mut [u8]) -> Result<usize, HandleError> {
        let handle_lock = self.handles.read(LockEncouragement::Weak);

        let Some(handle_info) = handle_lock.handles.get(&id) else {
            return Err(HandleError::HandleDoesntExist(id));
        };

        match handle_info {
            ProcessHandle::HostTwoWay {
                host_rx,
                host_tx: _,
                client,
                id: client_id,
            } => {
                let mut rx_lock = host_rx.write(LockEncouragement::Moderate);
                if rx_lock.len() == 0 {
                    if let Some(client_upgrade) = client.upgrade() {
                        client_upgrade
                            .signals
                            .write(LockEncouragement::Moderate)
                            .push_back(WaitSignal::HandleUpdate {
                                handle: *client_id,
                                kind: HandleUpdateKind::WriteReady,
                            });
                    }

                    return Err(HandleError::WouldBlock);
                }

                let mut bytes = 0;
                for entry_mut in data.iter_mut() {
                    let Some(front) = rx_lock.pop_front() else {
                        return Ok(bytes);
                    };

                    bytes += 1;
                    *entry_mut = front;
                }

                Ok(bytes)
            }
            ProcessHandle::ClientTwoWay { host, id } => {
                let host = host.upgrade().ok_or(HandleError::HostDisconnect)?;
                host.remote_rx(*id, data)
            }
            _ => Err(HandleError::InvalidSocketKind),
        }
    }

    /// Get the next wait signal for this process
    pub fn next_signal(&self) -> WaitSignal {
        loop {
            let len = { self.signals.read(LockEncouragement::Weak).len() };
            if len == 0 {
                Scheduler::yield_me();
            }

            match self.signals.write(LockEncouragement::Strong).pop_front() {
                Some(signal) => break signal,
                None => (),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleError {
    HandleDoesntExist(u64),
    InvalidSocketKind,
    HostDisconnect,
    WouldBlock,
}

impl Drop for Process {
    fn drop(&mut self) {
        let s = Scheduler::get();
        s.remove_process(self);
    }
}
