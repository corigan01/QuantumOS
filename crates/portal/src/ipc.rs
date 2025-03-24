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

pub type IpcString = alloc::string::String;
pub type IpcVec<T> = alloc::vec::Vec<T>;
pub type IpcResult<T> = ::core::result::Result<T, IpcError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpcError {
    InvalidMagic { given: u8, expected: u8 },
    BufferInvalidSize,
    Utf8ConvertError,
    InvalidTypeConvert,
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

pub const MESSAGE_SERVER_START: u8 = 0xF0;
pub const MESSAGE_CLIENT_START: u8 = 0xF8;

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

impl PortalConvert for () {
    fn serialize(&self, _send: &mut impl Sender) -> Result<usize, IpcError> {
        Ok(0)
    }

    fn deserialize(_recv: &mut impl Receiver) -> Result<Self, IpcError> {
        Ok(())
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

impl<T> PortalConvert for alloc::vec::Vec<T>
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

        let mut vec = alloc::vec::Vec::with_capacity(vec_len);
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

    impl Sender for Vec<u8> {
        fn send(&mut self, bytes: &[u8]) -> Result<(), IpcError> {
            self.extend_from_slice(bytes);
            Ok(())
        }
    }

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
