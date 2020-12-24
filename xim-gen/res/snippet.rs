// DO NOT EDIT YOURSELF
// This source is auto generated by xim-gen

#![allow(clippy::identity_op)]

use bstr::{BString, ByteSlice};
use std::convert::TryInto;

pub fn read<T>(b: &[u8]) -> Result<T, ReadError>
where
    T: XimRead,
{
    T::read(&mut Reader::new(b))
}

pub fn write<T>(val: T, out: &mut [u8])
where
    T: XimWrite,
{
    val.write(&mut Writer::new(out));
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Endian {
    #[cfg(target_endian = "little")]
    Native = 0x6c,
    #[cfg(target_endian = "big")]
    Native = 0x42,
    // Big = 0x42,
    // Little = 0x6c,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusContent {
    Text(StatusTextContent),
    Pixmap(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommitData {
    Keysym {
        keysym: u32,
        syncronous: bool,
    },
    Chars {
        commited: Vec<u8>,
        syncronous: bool,
    },
    Both {
        keysym: u32,
        commited: Vec<u8>,
        syncronous: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HotKeyTriggers {
    pub triggers: Vec<(TriggerKey, HotKeyState)>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("End of Stream")]
    EndOfStream,
    #[error("Invalid Data {0}: {1}")]
    InvalidData(&'static str, String),
    #[error("Not a native endian")]
    NotNativeEndian,
}

fn pad4(len: usize) -> usize {
    match len % 4 {
        0 => 0,
        x => 4 - x,
    }
}

fn with_pad4(len: usize) -> usize {
    len + pad4(len)
}

pub struct Reader<'b> {
    bytes: &'b [u8],
    start: usize,
}

impl<'b> Reader<'b> {
    pub fn new(bytes: &'b [u8]) -> Self {
        Self {
            bytes,
            start: bytes.as_ptr() as usize,
        }
    }

    fn ptr_offset(&self) -> usize {
        self.bytes.as_ptr() as usize - self.start
    }

    pub fn cursor(&self) -> usize {
        self.bytes.len()
    }

    pub fn pad4(&mut self) -> Result<(), ReadError> {
        self.consume(pad4(self.ptr_offset()))?;
        Ok(())
    }

    #[inline(always)]
    pub fn eos(&self) -> ReadError {
        ReadError::EndOfStream
    }

    pub fn invalid_data(&self, ty: &'static str, item: impl ToString) -> ReadError {
        ReadError::InvalidData(ty, item.to_string())
    }

    pub fn u8(&mut self) -> Result<u8, ReadError> {
        let (b, new) = self.bytes.split_first().ok_or(ReadError::EndOfStream)?;
        self.bytes = new;
        Ok(*b)
    }

    pub fn i16(&mut self) -> Result<i16, ReadError> {
        let bytes = self.consume(2)?.try_into().unwrap();
        Ok(i16::from_ne_bytes(bytes))
    }

    pub fn u16(&mut self) -> Result<u16, ReadError> {
        let bytes = self.consume(2)?.try_into().unwrap();
        Ok(u16::from_ne_bytes(bytes))
    }

    pub fn u32(&mut self) -> Result<u32, ReadError> {
        let bytes = self.consume(4)?.try_into().unwrap();
        Ok(u32::from_ne_bytes(bytes))
    }

    pub fn i32(&mut self) -> Result<i32, ReadError> {
        let bytes = self.consume(4)?.try_into().unwrap();
        Ok(i32::from_ne_bytes(bytes))
    }

    pub fn consume(&mut self, len: usize) -> Result<&'b [u8], ReadError> {
        if self.bytes.len() >= len {
            let (out, new) = self.bytes.split_at(len);
            self.bytes = new;
            Ok(out)
        } else {
            Err(self.eos())
        }
    }
}

pub struct Writer<'b> {
    out: &'b mut [u8],
    idx: usize,
}

impl<'b> Writer<'b> {
    pub fn new(out: &'b mut [u8]) -> Self {
        Self { out, idx: 0 }
    }

    pub fn write_u8(&mut self, b: u8) {
        self.out[self.idx] = b;
        self.idx += 1;
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.out[self.idx..self.idx + bytes.len()].copy_from_slice(bytes);
        self.idx += bytes.len();
    }

    pub fn write_pad4(&mut self) {
        let pad = pad4(self.idx);
        let pad_bytes = [0; 4];
        self.write(&pad_bytes[..pad]);
    }
}

pub trait XimRead: Sized {
    fn read(reader: &mut Reader) -> Result<Self, ReadError>;
}

pub trait XimWrite {
    fn write(&self, writer: &mut Writer);
    /// byte size of format
    fn size(&self) -> usize;
}

impl<'a, T> XimWrite for &'a T
where
    T: XimWrite,
{
    #[inline(always)]
    fn write(&self, writer: &mut Writer) {
        (**self).write(writer);
    }
    #[inline(always)]
    fn size(&self) -> usize {
        (**self).size()
    }
}

impl XimRead for Endian {
    fn read(reader: &mut Reader) -> Result<Self, ReadError> {
        let n = u8::read(reader)?;

        if n == Endian::Native as u8 {
            Ok(Self::Native)
        } else {
            Err(ReadError::NotNativeEndian)
        }
    }
}

impl XimWrite for Endian {
    fn write(&self, writer: &mut Writer) {
        (*self as u8).write(writer);
    }

    fn size(&self) -> usize {
        1
    }
}

impl XimRead for StatusContent {
    fn read(reader: &mut Reader) -> Result<Self, ReadError> {
        let ty = u32::read(reader)?;

        match ty {
            0 => Ok(Self::Text(StatusTextContent::read(reader)?)),
            1 => Ok(Self::Pixmap(u32::read(reader)?)),
            _ => Err(reader.invalid_data("StatusContentType", ty)),
        }
    }
}

impl XimWrite for StatusContent {
    fn write(&self, writer: &mut Writer) {
        match self {
            StatusContent::Text(content) => {
                0u32.write(writer);
                content.write(writer);
            }
            StatusContent::Pixmap(pixmap) => {
                1u32.write(writer);
                pixmap.write(writer);
            }
        }
    }

    fn size(&self) -> usize {
        let size = match self {
            StatusContent::Text(content) => content.size(),
            StatusContent::Pixmap(pixmap) => std::mem::size_of_val(pixmap),
        };

        size + 4
    }
}

impl XimRead for CommitData {
    fn read(reader: &mut Reader) -> Result<Self, ReadError> {
        let ty = reader.u16()?;

        match ty {
            2 | 3 => {
                let len = reader.u16()?;
                let bytes = reader.consume(len as usize)?;
                reader.pad4()?;
                Ok(Self::Chars {
                    commited: bytes.to_vec(),
                    syncronous: ty == 5,
                })
            }
            4 | 5 => {
                reader.consume(2)?;
                let keysym = reader.u32()?;
                Ok(Self::Keysym {
                    keysym,
                    syncronous: ty == 3,
                })
            }
            6 | 7 => {
                reader.consume(2)?;
                let keysym = reader.u32()?;
                let len = reader.u16()?;
                let bytes = reader.consume(len as usize)?;
                reader.pad4()?;
                Ok(Self::Both {
                    keysym,
                    commited: bytes.to_vec(),
                    syncronous: ty == 7,
                })
            }
            _ => Err(reader.invalid_data("CommitDataType", ty)),
        }
    }
}

impl XimWrite for CommitData {
    fn write(&self, writer: &mut Writer) {
        match self {
            Self::Chars {
                commited,
                syncronous,
            } => {
                let flag = if *syncronous { 3u16 } else { 2u16 };
                flag.write(writer);
                (commited.len() as u16).write(writer);
                writer.write(&commited);
                writer.write_pad4();
            }
            Self::Keysym { keysym, syncronous } => {
                let flag = if *syncronous { 5u16 } else { 4u16 };
                flag.write(writer);
                0u16.write(writer);
                keysym.write(writer);
            }
            Self::Both {
                keysym,
                commited,
                syncronous,
            } => {
                let flag = if *syncronous { 7u16 } else { 6u16 };
                flag.write(writer);
                0u16.write(writer);
                keysym.write(writer);
                (commited.len() as u16).write(writer);
                writer.write(&commited);
                writer.write_pad4();
            }
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Keysym { .. } => 6,
            Self::Chars { commited, .. } => with_pad4(commited.len() + 2),
            Self::Both { commited, .. } => with_pad4(commited.len() + 2) + 6,
        }
    }
}

impl XimRead for HotKeyTriggers {
    fn read(reader: &mut Reader) -> Result<Self, ReadError> {
        let n = reader.u32()? as usize;
        let mut out = Vec::with_capacity(n);

        for _ in 0..n {
            out.push((TriggerKey::read(reader)?, HotKeyState::Off));
        }

        for _ in 0..n {
            out[n].1 = HotKeyState::read(reader)?;
        }

        Ok(Self { triggers: out })
    }
}

impl XimWrite for HotKeyTriggers {
    fn write(&self, writer: &mut Writer) {
        (self.triggers.len() as u32).write(writer);

        for (trigger, _) in self.triggers.iter() {
            trigger.write(writer);
        }

        for (_, state) in self.triggers.iter() {
            state.write(writer);
        }
    }

    fn size(&self) -> usize {
        self.triggers.len() * 8 + 4
    }
}

impl XimRead for u8 {
    fn read(reader: &mut Reader) -> Result<Self, ReadError> {
        reader.u8()
    }
}

impl XimWrite for u8 {
    fn write(&self, writer: &mut Writer) {
        writer.write_u8(*self)
    }

    fn size(&self) -> usize {
        1
    }
}

impl XimRead for bool {
    fn read(reader: &mut Reader) -> Result<Self, ReadError> {
        Ok(reader.u8()? != 0)
    }
}

impl XimWrite for bool {
    fn write(&self, writer: &mut Writer) {
        writer.write_u8(*self as u8)
    }

    fn size(&self) -> usize {
        1
    }
}

macro_rules! impl_int {
    ($ty:ident) => {
        impl XimRead for $ty {
            fn read(reader: &mut Reader) -> Result<Self, ReadError> {
                reader.$ty()
            }
        }

        impl XimWrite for $ty {
            fn write(&self, writer: &mut Writer) {
                writer.write(&self.to_ne_bytes())
            }

            fn size(&self) -> usize {
                std::mem::size_of::<$ty>()
            }
        }
    };
}

impl_int!(u16);
impl_int!(i16);
impl_int!(u32);
impl_int!(i32);
