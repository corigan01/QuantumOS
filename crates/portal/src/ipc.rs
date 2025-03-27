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

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use convert::{
    MESSAGE_CLIENT_REQ_START, MESSAGE_CLIENT_RSP_START, MESSAGE_END, MESSAGE_SERVER_REQ_START,
    MESSAGE_SERVER_RSP_START,
};
use core::marker::PhantomData;

pub mod convert;

pub type IpcString = alloc::string::String;
pub type IpcVec<T> = Vec<T>;
pub type IpcResult<T> = ::core::result::Result<T, IpcError>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpcError {
    InvalidMagic { given: u8, expected: u8 },
    GlueError,
    BufferInvalidSize,
    Utf8ConvertError,
    InvalidTypeConvert,
    NotReady,
    InvalidMessage(Vec<u8>),
    InvalidHash { given: u64, expected: u64 },
}

/// Ipc Sender (TX)
///
/// This trait supports writting bytes over IPC.
pub trait Sender {
    fn send(&mut self, bytes: &[u8]) -> IpcResult<()>;
}

/// Ipc Receiver (RX)
///
/// This trait supports reading bytes from IPC.
pub trait Receiver {
    fn recv(&mut self, bytes: &mut [u8]) -> IpcResult<usize>;

    fn recv_exact(&mut self, bytes: &mut [u8]) -> IpcResult<()> {
        match self.recv(bytes) {
            Ok(len) if len == bytes.len() => Ok(()),
            Ok(_) => Err(IpcError::BufferInvalidSize),
            Err(err) => Err(err),
        }
    }
}

/// A typed sender for responding to IPC Messages
pub struct IpcResponder<
    'a,
    Glue: IpcGlue,
    Info: IpcServiceInfo,
    T: PortalConvert,
    const TARGET_ID: u64,
> {
    connection: &'a mut IpcService<Glue, Info>,
    ty: PhantomData<T>,
}

impl<'a, Glue: IpcGlue, Info: IpcServiceInfo, T: PortalConvert, const TARGET_ID: u64>
    IpcResponder<'a, Glue, Info, T, TARGET_ID>
{
    pub fn new(connection: &'a mut IpcService<Glue, Info>) -> Self {
        Self {
            connection,
            ty: PhantomData,
        }
    }

    pub fn respond_with(self, value: T) -> IpcResult<()> {
        self.connection.tx_msg(TARGET_ID, true, value)
    }
}

/// Conversion from/to IPC Sockets
///
/// This trait allows a type to be converted into and from the IPC
/// portal. It is required that all types transfered over IPC implement
/// this trait.
///
/// # Note
/// Users of a portal should **not** implement this themselves. Always use
/// the portal macro.
pub trait PortalConvert: Sized {
    /// Serialize this type for transfer
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError>;
    /// Deserialize the transfered bytes into `Self`
    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError>;
}

/// A header-valid IPC message
///
/// This structure represents an ipc message with a valid header. This structure
/// does not, however, verify the contents of `data` -- during parsing that is
/// further converted into the final type.
#[derive(Debug)]
pub struct IpcMessage {
    pub start_byte: u8,
    pub endpoint_hash: u64,
    pub target_id: u64,
    pub data: Vec<u8>,
    pub end_byte: u8,
}

impl IpcMessage {
    /// Try and parse this message into a valid convertable type.
    pub fn try_parse<T: PortalConvert>(&self) -> IpcResult<T> {
        let mut data_ref = self.data.as_slice();
        T::deserialize(&mut data_ref)
    }
}

/// A unknown IPC message(s) whos contents are in-progress of being parsed.
pub struct RawIpcBuffer(Vec<u8>);

impl RawIpcBuffer {
    /// Create a new empty IPC buffer
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Add bytes to the end of the stream
    pub fn append(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    /// Get the starting byte of the message
    ///
    /// # Note
    /// This function does not verify the rest of the message, only that the
    /// start byte is one of the four possible values.
    pub fn get_start_byte(&self) -> IpcResult<u8> {
        self.0
            .get(0)
            .map(|&byte| match byte {
                MESSAGE_SERVER_RSP_START
                | MESSAGE_SERVER_REQ_START
                | MESSAGE_CLIENT_RSP_START
                | MESSAGE_CLIENT_REQ_START => Ok(byte),
                _ => Err(IpcError::InvalidTypeConvert),
            })
            .ok_or(IpcError::NotReady)?
    }

    pub fn get_endpoint_hash(&self) -> IpcResult<u64> {
        let mut endpoint_slice = self.0.get(1..10).ok_or(IpcError::NotReady)?;
        u64::deserialize(&mut endpoint_slice)
    }

    pub fn get_target_id(&self) -> IpcResult<u64> {
        let mut target_slice = self.0.get(10..19).ok_or(IpcError::NotReady)?;
        u64::deserialize(&mut target_slice)
    }

    pub fn get_data_len(&self) -> IpcResult<usize> {
        let mut len_slice = self.0.get(19..28).ok_or(IpcError::NotReady)?;
        Ok(u64::deserialize(&mut len_slice)? as usize)
    }

    pub fn get_data(&self) -> IpcResult<Vec<u8>> {
        let data_start = 28;
        let data_end = data_start + self.get_data_len()?;

        Ok(self
            .0
            .get(data_start..data_end)
            .ok_or(IpcError::NotReady)?
            .into())
    }

    pub fn get_end_byte(&self) -> IpcResult<u8> {
        let data_len = self.get_data_len()?;
        let end_index = 28 + data_len;

        self.0
            .get(end_index)
            .map(|&byte| match byte {
                MESSAGE_END => Ok(byte),
                _ => Err(IpcError::InvalidTypeConvert),
            })
            .ok_or(IpcError::NotReady)?
    }

    fn populate_ipc_message(&self) -> IpcResult<IpcMessage> {
        Ok(IpcMessage {
            start_byte: self.get_start_byte()?,
            endpoint_hash: self.get_endpoint_hash()?,
            target_id: self.get_target_id()?,
            data: self.get_data()?,
            end_byte: self.get_end_byte()?,
        })
    }

    /// Get and pop this message off the message stream
    ///
    /// # Note
    /// This will not always remove data from the data stream. For example, if the
    /// status of the convert was `IpcError::NotReady` then the stream will simply
    /// just return `NotReady` until more data bytes arrive.
    ///
    /// It is best to disconnect when the client sees data stream errors, however, the
    /// design of the protocol should make it mostly recoverable in this event.
    pub fn pop_message(&mut self) -> IpcResult<IpcMessage> {
        match self.populate_ipc_message() {
            Err(IpcError::NotReady) => Err(IpcError::NotReady),
            Ok(valid) => {
                self.0.drain(0..=valid.data.len() + 29);
                Ok(valid)
            }
            Err(invalid) => {
                // FIXME: Make a better impl for fixing the data stream
                let remove_end = self
                    .0
                    .iter()
                    .enumerate()
                    .find_map(|(i, &byte)| match byte {
                        MESSAGE_SERVER_RSP_START
                        | MESSAGE_SERVER_REQ_START
                        | MESSAGE_CLIENT_RSP_START
                        | MESSAGE_CLIENT_REQ_START => Some(i),
                        MESSAGE_END => Some(i + 1),
                        _ => None,
                    })
                    .unwrap_or(self.0.len());
                self.0.drain(0..remove_end);

                return Err(invalid);
            }
        }
    }
}

/// Info for a given endpoint service required for connection
///
/// This trait serves to provide a constant way of storing the
/// endpoint's info for the inner `IpcService` struct.
pub trait IpcServiceInfo {
    const ENDPOINT_NAME: &'static str;
    const ENDPOINT_HASH: u64;
}

pub trait IpcGlue: Sender + Receiver {
    /// Disconnect Portal's connection from server and client
    ///
    /// Calls to `disconnect` should cleanly disconnect the backing
    /// communication of this `IpcGlue` structure.
    fn disconnect(&mut self);

    /// If the ipc backend is blocking during a waiting call, it will
    /// call this method to block until the socket has woken up.
    fn socket_wait(&self) {}

    // /// Make a connection to the server provided with the service info
    // fn connect<Info: IpcServiceInfo>(&mut self) -> IpcResult<()>;

    // /// Begin hosting a connection provided with the service info
    // fn begin_serve<Info: IpcServiceInfo>(&mut self) -> IpcResult<()>;
}

pub struct IpcService<Glue: IpcGlue, Info: IpcServiceInfo> {
    glue: Glue,
    info: PhantomData<Info>,
    is_server: bool,
    rx_queue: VecDeque<IpcMessage>,
    tx_queue: VecDeque<IpcMessage>,
    rx_buf: RawIpcBuffer,
}

impl<Glue: IpcGlue, Info: IpcServiceInfo> IpcService<Glue, Info> {
    pub fn new(glue: Glue, is_server: bool) -> Self {
        Self {
            glue,
            info: PhantomData,
            rx_queue: VecDeque::new(),
            tx_queue: VecDeque::new(),
            rx_buf: RawIpcBuffer::new(),
            is_server,
        }
    }

    /// Try to read data into the data queue and parse it into `IpcMessage`'s
    pub fn drive_rx(&mut self) -> IpcResult<()> {
        // read into the data queue
        loop {
            let mut data_chunk = [0; 256];

            match self.glue.recv(&mut data_chunk) {
                Ok(0) | Err(IpcError::BufferInvalidSize) | Err(IpcError::NotReady) => break,
                Ok(valid_len) => {
                    self.rx_buf.append(&data_chunk[..valid_len]);
                }
                Err(other) => return Err(other),
            }
        }

        // try to parse messages
        loop {
            match self.rx_buf.pop_message() {
                Ok(valid) => {
                    if valid.endpoint_hash != Info::ENDPOINT_HASH {
                        return Err(IpcError::InvalidHash {
                            given: valid.endpoint_hash,
                            expected: Info::ENDPOINT_HASH,
                        });
                    }

                    self.rx_queue.push_back(valid);
                }
                Err(IpcError::NotReady) => break Ok(()),
                Err(invalid) => return Err(invalid),
            }
        }
    }

    /// A blocking RX and deserialization call to the service
    pub fn blocking_rx<T: PortalConvert>(&mut self, target_id: u64) -> IpcResult<T> {
        let is_server = self.is_server;

        loop {
            self.drive_rx()?;

            if self.rx_queue.is_empty() {
                self.glue.socket_wait();
                continue;
            }

            if let Some(reponse) = self.pop_rx_if(|messages| {
                messages.target_id == target_id
                    && messages.start_byte
                        == if is_server {
                            MESSAGE_CLIENT_RSP_START
                        } else {
                            MESSAGE_SERVER_RSP_START
                        }
            }) {
                return T::deserialize(&mut reponse.data.as_slice());
            }
        }
    }

    /// Construct a new IpcMessage and add it to the transmit queue
    pub fn tx_msg<T: PortalConvert>(
        &mut self,
        target_id: u64,
        is_response: bool,
        data: T,
    ) -> IpcResult<()> {
        let start_byte = match () {
            _ if self.is_server && is_response => MESSAGE_SERVER_RSP_START,
            _ if self.is_server => MESSAGE_SERVER_REQ_START,
            _ if is_response => MESSAGE_CLIENT_RSP_START,
            _ => MESSAGE_CLIENT_REQ_START,
        };

        let mut data_vec = Vec::with_capacity(256);
        data.serialize(&mut data_vec)?;

        self.tx_queue.push_back(IpcMessage {
            start_byte,
            endpoint_hash: Info::ENDPOINT_HASH,
            target_id,
            data: data_vec,
            end_byte: MESSAGE_END,
        });

        Ok(())
    }

    /// Serialize all messages into one large vector, then call `send` from `glue`.
    ///
    /// # Note
    /// This will call multiple times
    pub fn flush_tx(&mut self) -> IpcResult<()> {
        let mut holding_cell = Vec::with_capacity(256);

        for ipc_message in self.tx_queue.drain(..) {
            // If our length is too large, we want to flush the array to glue
            if holding_cell.len() >= (8 * 1024) - 1 {
                self.glue.send(&holding_cell)?;
                holding_cell.clear();
            }

            ipc_message.serialize(&mut holding_cell)?;
        }

        self.glue.send(&holding_cell)?;
        Ok(())
    }

    pub fn pop_rx(&mut self) -> Option<IpcMessage> {
        self.rx_queue.pop_front()
    }

    pub fn pop_rx_if<F>(&mut self, mut f: F) -> Option<IpcMessage>
    where
        F: FnMut(&IpcMessage) -> bool,
    {
        let index = self
            .rx_queue
            .iter()
            .enumerate()
            .find_map(|(i, message)| f(message).then_some(i))?;

        self.rx_queue.remove(index)
    }
}
