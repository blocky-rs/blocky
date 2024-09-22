use std::io::Write;

use blocky_world::position::{BlockPosition, ChunkPosition};
use uuid::Uuid;

use crate::types::VarInt;

pub trait Encoder {
    fn byte_len(&self) -> usize
    where
        Self: Sized;

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()>
    where
        Self: Sized;

    fn to_bytes(&self) -> anyhow::Result<Vec<u8>>
    where
        Self: Sized,
    {
        let mut buf = Vec::with_capacity(self.byte_len());
        self.encode(&mut buf)?;
        Ok(buf)
    }
}

impl<V: Encoder> Encoder for Option<V> {
    fn byte_len(&self) -> usize {
        match self {
            Some(v) => v.byte_len() + 1,
            None => 1,
        }
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        match self {
            Some(value) => {
                bool::encode(&true, buf)?;
                value.encode(buf)
            }
            None => bool::encode(&false, buf),
        }
    }
}

impl Encoder for ChunkPosition {
    fn byte_len(&self) -> usize {
        8
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        i32::encode(&self.x, buf)?;
        i32::encode(&self.z, buf)?;
        Ok(())
    }
}

impl Encoder for BlockPosition {
    fn byte_len(&self) -> usize {
        8
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        let value = ((self.x as u64 & 0x3FFFFFF) << 38)
            | ((self.z as u64 & 0x3FFFFFF) << 12)
            | (self.y as u64 & 0xFFF);

        u64::encode(&value, buf)
    }
}

impl Encoder for bool {
    fn byte_len(&self) -> usize {
        1
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        buf.write_all(&[*self as u8])?;
        Ok(())
    }
}

impl Encoder for String {
    fn byte_len(&self) -> usize {
        VarInt(self.len() as i32).byte_len() + self.len()
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        let length = self.len();

        let max_length = i16::MAX as usize;
        if length > max_length {
            anyhow::bail!(
                "String length {} exceeds maximum allowed length of {}",
                length,
                max_length
            );
        }

        VarInt(length as i32).encode(buf)?;
        buf.write_all(&self.to_bytes()?)?;
        Ok(())
    }
}

impl Encoder for Uuid {
    fn byte_len(&self) -> usize {
        16
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        self.as_u128().encode(buf)?;
        Ok(())
    }
}

macro_rules! impl_number_encoder {
    ($($typ:ty),* $(,)?) => {
        $(
            impl Encoder for $typ {
                fn byte_len(&self) -> usize {
                    std::mem::size_of::<Self>()
                }

                fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
                    buf.write_all(&self.to_be_bytes())?;
                    Ok(())
                }
            }

            impl super::types::LengthPrefix for $typ {
                fn len(&self) -> usize {
                    *self as usize
                }

                fn from_len(value: usize) -> Self {
                    value as $typ
                }
            }
        )*
    };
}

impl_number_encoder!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
