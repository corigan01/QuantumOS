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

use alloc::{collections::VecDeque, vec::Vec};

extern crate alloc;

pub type IpcString = alloc::string::String;
pub type IpcVec<T> = Vec<T>;
pub type IpcResult<T> = ::core::result::Result<T, IpcError>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpcError {
    InvalidMagic { given: u8, expected: u8 },
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
    fn send(&mut self, bytes: &[u8]) -> Result<(), IpcError>;
}

/// Ipc Receiver (RX)
///
/// This trait supports reading bytes from IPC.
pub trait Receiver {
    fn recv(&mut self, bytes: &mut [u8]) -> Result<(), IpcError>;
}

/// A typed sender for responding to IPC Messages
pub struct IpcResponder<'a, S: Sender, T: PortalConvert> {
    sender: &'a mut S,
    ty: PhantomData<T>,
}

impl<'a, S: Sender, T: PortalConvert> IpcResponder<'a, S, T> {
    /// Construct a new IpcResponder for a given type
    pub const fn new(sender: &'a mut S) -> Self {
        Self {
            sender,
            ty: PhantomData,
        }
    }

    /// Respond with a give value
    pub fn respond_with(self, value: T) -> Result<usize, IpcError> {
        value.serialize(self.sender)
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

/// A message queue cache for out of order messages during blocking requests
pub struct MessageCache {
    queue: VecDeque<Vec<u8>>,
}

impl MessageCache {
    /// Create a new empty cache
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Put this message onto the back of the queue
    pub fn queue_message(&mut self, message: &[u8]) {
        self.queue.push_back(message.into());
    }

    /// Put this message into the back of the queue from a vec
    pub fn queue_vec(&mut self, message: Vec<u8>) {
        self.queue.push_back(message);
    }

    /// Get a mut ref to the last element in the array
    pub fn last_message_mut<'a>(&'a mut self) -> Option<&'a mut Vec<u8>> {
        self.queue.back_mut()
    }

    /// Get the next message in the queue
    pub fn next_message(&mut self) -> Option<Vec<u8>> {
        self.queue.pop_front()
    }

    /// Peek the next message in the queue
    pub fn peek_next<'a>(&'a self) -> Option<&'a Vec<u8>> {
        self.queue.front()
    }
}

/// This is the actual impl that will send data over the IPC socket
pub trait TxRxGlue {
    /// Get the MessageCache for the RX buffer
    fn rx_cache<'a>(&mut self) -> &'a mut MessageCache;

    /// Get the MessageCache for the TX buffer
    fn tx_cache<'a>(&mut self) -> &'a mut MessageCache;

    /// Flush the TX buffer over the IPC socket
    fn flush_tx(&mut self) -> Result<(), IpcError>;

    /// Read data into the RX buffer
    fn peek_rx_buffer<'a>(&'a self) -> Result<&'a [u8], IpcError>;
    fn take_rx_buffer(&mut self) -> Vec<u8>;
}

pub trait IpcDevice: TxRxGlue {
    const ENDPOINT_HASH: u64;
    const IS_SERVER: bool;

    fn message_send<T: PortalConvert>(
        &mut self,
        is_reponse: bool,
        target_id: u64,
        data: &T,
    ) -> Result<usize, IpcError> {
        let send_byte = match () {
            _ if !is_reponse && Self::IS_SERVER => MESSAGE_SERVER_REQ_START,
            _ if is_reponse && Self::IS_SERVER => MESSAGE_SERVER_RSP_START,
            _ if !is_reponse => MESSAGE_CLIENT_REQ_START,
            _ => MESSAGE_CLIENT_RSP_START,
        };

        let mut message = Vec::with_capacity(128);

        // 1. Start Byte
        message.push(send_byte);

        // 2. Endpoint ID for this IPC kind
        Self::ENDPOINT_HASH.serialize(&mut message)?;

        // 3. ID for which the message is to be recv
        target_id.serialize(&mut message)?;

        // 4. Serialize the message
        data.serialize(&mut message)?;

        // 5. End the transmission
        message.push(MESSAGE_END);

        // Send the message
        let message_len = message.len();
        self.tx_cache().queue_vec(message);
        self.flush_tx()?;

        Ok(message_len)
    }
}

impl Sender for Vec<u8> {
    fn send(&mut self, bytes: &[u8]) -> Result<(), IpcError> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}

impl Receiver for Option<&[u8]> {
    fn recv(&mut self, bytes: &mut [u8]) -> Result<(), IpcError> {
        let Some(inner) = self else {
            return Err(IpcError::BufferInvalidSize);
        };

        if inner.len() < bytes.len() {
            return Err(IpcError::BufferInvalidSize);
        }

        bytes.copy_from_slice(&inner[..bytes.len()]);
        Ok(())
    }
}

pub const MESSAGE_SERVER_REQ_START: u8 = 0xF0;
pub const MESSAGE_SERVER_RSP_START: u8 = 0xF1;
pub const MESSAGE_CLIENT_REQ_START: u8 = 0xF8;
pub const MESSAGE_CLIENT_RSP_START: u8 = 0xF9;

pub const MESSAGE_END: u8 = 0xFF;

pub const CONVERT_U8: u8 = 1;
pub const CONVERT_U16: u8 = 2;
pub const CONVERT_U32: u8 = 3;
pub const CONVERT_U64: u8 = 4;

pub const CONVERT_I8: u8 = 5;
pub const CONVERT_I16: u8 = 6;
pub const CONVERT_I32: u8 = 7;
pub const CONVERT_I64: u8 = 8;

pub const CONVERT_STR: u8 = 9;
pub const CONVERT_VEC: u8 = 10;
pub const CONVERT_TAG: u8 = 11;
pub const CONVERT_UNIT: u8 = 12;

impl PortalConvert for () {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        // Unit needs to send data because it is often used as 'non-data' signals.
        //
        // For example, if you are making a blocking request that doesn't return anything
        // the receiver still needs to be informed when the request finished.
        send.send(&[CONVERT_UNIT])?;
        Ok(1)
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut data_buffer = [0];
        recv.recv(&mut data_buffer)?;

        if data_buffer[0] != CONVERT_UNIT {
            return Err(IpcError::InvalidMagic {
                given: data_buffer[0],
                expected: CONVERT_UNIT,
            });
        }

        Ok(())
    }
}

impl PortalConvert for u8 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        let array = [CONVERT_U8, *self];
        send.send(&array)?;
        Ok(array.len())
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_U8 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_U8,
            });
        }

        Ok(recv_array[1])
    }
}

impl PortalConvert for u16 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_U16])?;
        send.send(&self.to_ne_bytes())?;
        Ok(const { (u16::BITS as usize / 8) + 1 })
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_U16 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_U16,
            });
        }

        Ok(u16::from_ne_bytes(
            recv_array[1..3]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        ))
    }
}

impl PortalConvert for u32 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_U32])?;
        send.send(&self.to_ne_bytes())?;
        Ok(const { (u32::BITS as usize / 8) + 1 })
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0, 0, 0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_U32 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_U32,
            });
        }

        Ok(u32::from_ne_bytes(
            recv_array[1..5]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        ))
    }
}

impl PortalConvert for u64 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_U64])?;
        send.send(&self.to_ne_bytes())?;
        Ok(const { (u64::BITS as usize / 8) + 1 })
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_U64 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_U64,
            });
        }

        Ok(u64::from_ne_bytes(
            recv_array[1..9]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        ))
    }
}

impl PortalConvert for i8 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        let array = [CONVERT_I8, *self as u8];
        send.send(&array)?;
        Ok(array.len())
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_I8 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_I8,
            });
        }

        Ok(recv_array[1] as i8)
    }
}

impl PortalConvert for i16 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_I16])?;
        send.send(&self.to_ne_bytes())?;
        Ok(const { (i16::BITS as usize / 8) + 1 })
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_I16 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_I16,
            });
        }

        Ok(i16::from_ne_bytes(
            recv_array[1..3]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        ))
    }
}

impl PortalConvert for i32 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_I32])?;
        send.send(&self.to_ne_bytes())?;
        Ok(const { (i32::BITS as usize / 8) + 1 })
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0, 0, 0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_I32 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_I32,
            });
        }

        Ok(i32::from_ne_bytes(
            recv_array[1..5]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        ))
    }
}

impl PortalConvert for i64 {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_I64])?;
        send.send(&self.to_ne_bytes())?;
        Ok(const { (i64::BITS as usize / 8) + 1 })
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_I64 {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_I64,
            });
        }

        Ok(i64::from_ne_bytes(
            recv_array[1..9]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        ))
    }
}

impl PortalConvert for usize {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        (*self as u64).serialize(send)
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        Ok(u64::deserialize(recv)? as usize)
    }
}

impl PortalConvert for isize {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        (*self as i64).serialize(send)
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        Ok(i64::deserialize(recv)? as isize)
    }
}

impl<T> PortalConvert for Option<T>
where
    T: PortalConvert,
{
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        match self {
            Some(inner) => {
                send.send(&[CONVERT_TAG, 1])?;
                Ok(inner.serialize(send)? + 2)
            }
            None => {
                send.send(&[CONVERT_TAG, 0])?;
                Ok(2)
            }
        }
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_TAG {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_TAG,
            });
        }

        match recv_array[1] {
            0 => Ok(None),
            1 => Ok(Some(T::deserialize(recv)?)),
            _ => Err(IpcError::InvalidTypeConvert),
        }
    }
}

impl<O, E> PortalConvert for Result<O, E>
where
    O: PortalConvert,
    E: PortalConvert,
{
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        match self {
            Ok(inner) => {
                send.send(&[CONVERT_TAG, 2])?;
                Ok(inner.serialize(send)? + 2)
            }
            Err(inner) => {
                send.send(&[CONVERT_TAG, 3])?;
                Ok(inner.serialize(send)? + 2)
            }
        }
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        let mut recv_array = [0, 0];
        recv.recv(&mut recv_array)?;

        if recv_array[0] != CONVERT_TAG {
            return Err(IpcError::InvalidMagic {
                given: recv_array[0],
                expected: CONVERT_TAG,
            });
        }

        match recv_array[1] {
            2 => Ok(Ok(O::deserialize(recv)?)),
            3 => Ok(Err(E::deserialize(recv)?)),
            _ => Err(IpcError::InvalidTypeConvert),
        }
    }
}

impl PortalConvert for alloc::string::String {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_STR])?;
        send.send(&self.len().to_ne_bytes())?;
        send.send(self.as_bytes())?;

        Ok(1 + self.len() + (usize::BITS as usize / 8))
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        #[cfg(target_pointer_width = "64")]
        let mut magic_len = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        #[cfg(target_pointer_width = "32")]
        let mut magic_len = [0, 0, 0, 0, 0];
        recv.recv(&mut magic_len)?;

        if magic_len[0] != CONVERT_STR {
            return Err(IpcError::InvalidMagic {
                given: magic_len[0],
                expected: CONVERT_STR,
            });
        }

        let str_len = usize::from_ne_bytes(
            magic_len[1..]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        );

        let mut empty_slice = alloc::vec![0; str_len];
        recv.recv(&mut empty_slice)?;

        Ok(
            alloc::string::String::from_utf8(empty_slice)
                .map_err(|_| IpcError::Utf8ConvertError)?,
        )
    }
}

impl PortalConvert for bool {
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        (*self as u8).serialize(send)
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        match u8::deserialize(recv)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(IpcError::InvalidTypeConvert),
        }
    }
}

impl<T> PortalConvert for Vec<T>
where
    T: PortalConvert,
{
    fn serialize(&self, send: &mut impl Sender) -> Result<usize, IpcError> {
        send.send(&[CONVERT_VEC])?;
        send.send(&self.len().to_ne_bytes())?;

        let mut bytes = 1 + (usize::BITS as usize / 8);

        for item in self {
            bytes += item.serialize(send)?;
        }

        Ok(bytes)
    }

    fn deserialize(recv: &mut impl Receiver) -> Result<Self, IpcError> {
        #[cfg(target_pointer_width = "64")]
        let mut magic_len = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        #[cfg(target_pointer_width = "32")]
        let mut magic_len = [0, 0, 0, 0, 0];
        recv.recv(&mut magic_len)?;

        if magic_len[0] != CONVERT_VEC {
            return Err(IpcError::InvalidMagic {
                given: magic_len[0],
                expected: CONVERT_VEC,
            });
        }

        let vec_len = usize::from_ne_bytes(
            magic_len[1..]
                .try_into()
                .map_err(|_| IpcError::BufferInvalidSize)?,
        );

        let mut vec = Vec::with_capacity(vec_len);
        for _ in 0..vec_len {
            vec.push(T::deserialize(recv)?);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    impl Receiver for Vec<u8> {
        fn recv(&mut self, bytes: &mut [u8]) -> Result<(), IpcError> {
            for byte in bytes.iter_mut() {
                *byte = self.remove(0);
            }
            Ok(())
        }
    }

    #[test]
    fn test_simple_primitive() {
        let mut dummy = Vec::new();

        for sample in 0..3 {
            sample.serialize(&mut dummy).unwrap();
        }

        for sample in 0..3 {
            assert_eq!(i32::deserialize(&mut dummy), Ok(sample));
        }
    }

    #[test]
    fn test_complex() {
        let mut dummy = Vec::new();

        let correct = vec![
            vec![String::from("hello"), String::from("world")],
            vec![String::from("hello"), String::from("world")],
        ];
        assert_eq!(correct.serialize(&mut dummy), Ok(dummy.len()));

        let test_output: Vec<Vec<String>> = Vec::deserialize(&mut dummy).unwrap();
        assert_eq!(test_output, correct);

        assert_eq!(correct.serialize(&mut dummy), Ok(dummy.len()));
        let fail_output: Result<Vec<Vec<u64>>, IpcError> = Vec::deserialize(&mut dummy);
        assert_eq!(
            fail_output,
            Err(IpcError::InvalidMagic {
                given: CONVERT_STR,
                expected: CONVERT_U64
            })
        );
    }

    #[test]
    fn test_enum_complex() {
        let mut dummy = Vec::new();

        let my_option = Some(10);
        my_option.serialize(&mut dummy).unwrap();

        let my_result_ok: Result<Option<i32>, usize> = Ok(Some(10));
        my_result_ok.serialize(&mut dummy).unwrap();
        let my_result_err: Result<Option<i32>, usize> = Err(100);
        my_result_err.serialize(&mut dummy).unwrap();

        assert_eq!(Option::<i32>::deserialize(&mut dummy).unwrap(), my_option);
        assert_eq!(
            Result::<Option<i32>, usize>::deserialize(&mut dummy).unwrap(),
            my_result_ok
        );
        assert_eq!(
            Result::<Option<i32>, usize>::deserialize(&mut dummy).unwrap(),
            my_result_err
        );
    }
}
