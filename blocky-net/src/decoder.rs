use std::io::{Cursor, Read};

use blocky_world::position::{BlockPosition, ChunkPosition};
use uuid::Uuid;

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

impl<V: Decoder> Decoder for Option<V> {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let present = bool::decode(buf)?;

        if present {
            Ok(Some(V::decode(buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Decoder for ChunkPosition {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let x = i32::decode(buf)?;
        let z = i32::decode(buf)?;

        Ok(Self { x, z })
    }
}

impl Decoder for BlockPosition {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let val = i64::decode(buf)?;

        let x = (val >> 38) as i32;
        let y = (val & 0xFFF) as i32;
        let z = (val << 26 >> 38) as i32;

        Ok(Self { x, y, z })
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

impl Decoder for Uuid {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let value = u128::decode(buf)?;
        Ok(Uuid::from_u128(value))
    }
}

macro_rules! impl_number_decoder {
    ($($typ:ty),* $(,)?) => {
        $(
            impl Decoder for $typ {
                fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
                    let mut bytes = [0; std::mem::size_of::<Self>()];
                    buf.read_exact(&mut bytes)?;
                    Ok(Self::from_be_bytes(bytes))
                }
            }
        )*
    };
}

impl_number_decoder!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
