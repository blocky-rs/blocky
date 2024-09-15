use std::io::{Cursor, Read};

use crate::types::VarInt;

pub trait Decoder {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(bytes);
        Self::decode(&mut cursor)
    }
}

impl Decoder for bool {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let mut byte = [0];
        buf.read_exact(&mut byte)?;
        let byte = byte[0];

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => anyhow::bail!("Invalid boolean value"),
        }
    }
}

impl Decoder for String {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let length = VarInt::decode(buf)?.0 as usize;

        let max_length = i16::MAX as usize;
        if length > max_length {
            anyhow::bail!(
                "String length {} exceeds maximum allowed length of {}",
                length,
                max_length
            );
        }

        let mut bytes = vec![0; length];
        buf.read_exact(&mut bytes)?;

        let s = std::str::from_utf8(&bytes)?;
        Ok(s.to_string())
    }
}

macro_rules! impl_number_decoder {
    ($typ:ty) => {
        impl Decoder for $typ {
            fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
                let mut bytes = [0; std::mem::size_of::<Self>()];
                buf.read_exact(&mut bytes)?;
                Ok(Self::from_be_bytes(bytes))
            }
        }
    };
}

impl_number_decoder!(u8);
impl_number_decoder!(u16);
impl_number_decoder!(u32);
impl_number_decoder!(u64);
impl_number_decoder!(u128);

impl_number_decoder!(i8);
impl_number_decoder!(i16);
impl_number_decoder!(i32);
impl_number_decoder!(i64);
impl_number_decoder!(i128);

impl_number_decoder!(f32);
impl_number_decoder!(f64);
