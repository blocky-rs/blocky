use std::io::Write;

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

impl Encoder for bool {
    fn byte_len(&self) -> usize {
        1
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        buf.write(&[*self as u8])?;
        Ok(())
    }
}

impl Encoder for String {
    fn byte_len(&self) -> usize {
        VarInt(self.len() as i32).byte_len() + self.len()
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        let length = self.len();

        let max_length = std::i16::MAX as usize;
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

macro_rules! impl_number_encoder {
    ($typ:ty) => {
        impl Encoder for $typ {
            fn byte_len(&self) -> usize {
                std::mem::size_of::<Self>()
            }

            fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
                buf.write_all(&self.to_be_bytes())?;
                Ok(())
            }
        }
    };
}

impl_number_encoder!(u8);
impl_number_encoder!(u16);
impl_number_encoder!(u32);
impl_number_encoder!(u64);
impl_number_encoder!(u128);

impl_number_encoder!(i8);
impl_number_encoder!(i16);
impl_number_encoder!(i32);
impl_number_encoder!(i64);
impl_number_encoder!(i128);

impl_number_encoder!(f32);
impl_number_encoder!(f64);
