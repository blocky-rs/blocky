use std::{
    io::{Read, Write},
    iter,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{decoder::Decoder, encoder::Encoder};

pub struct LengthInferredVecU8(pub Vec<u8>);

impl Decoder for LengthInferredVecU8 {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let mut v = Vec::new();
        buf.read_to_end(&mut v)?;
        Ok(Self(v))
    }
}

impl Encoder for LengthInferredVecU8 {
    fn byte_len(&self) -> usize {
        self.0.len()
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        buf.write_all(&self.0)?;
        Ok(())
    }
}

pub const MAX_LENGTH: usize = 1024 * 1024;

pub trait LengthPrefix {
    fn len(&self) -> usize;
    fn from_len(value: usize) -> Self;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct LengthPrefixedVecU8<L>(pub Vec<u8>, PhantomData<L>)
where
    L: Encoder + Decoder + LengthPrefix;

impl<L> Decoder for LengthPrefixedVecU8<L>
where
    L: Encoder + Decoder + LengthPrefix,
{
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let len = L::decode(buf)?.len();
        let mut data = vec![0; len];
        buf.read_exact(&mut data)?;
        Ok(Self(data, PhantomData))
    }
}

impl<L> Encoder for LengthPrefixedVecU8<L>
where
    L: Encoder + Decoder + LengthPrefix,
{
    fn byte_len(&self) -> usize {
        L::from_len(self.0.len()).byte_len() + self.0.len()
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        L::from_len(self.0.len()).encode(buf)?;
        buf.write_all(&self.0)?;
        Ok(())
    }
}

pub struct LengthPrefixedVec<L, V>(pub Vec<V>, PhantomData<L>)
where
    L: Encoder + Decoder + LengthPrefix,
    V: Encoder + Decoder;

impl<L, V> Decoder for LengthPrefixedVec<L, V>
where
    L: Encoder + Decoder + LengthPrefix,
    V: Encoder + Decoder,
{
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let len = L::decode(buf)?.len();
        if len > MAX_LENGTH {
            anyhow::bail!("Length exceeds maximum allowed length");
        }

        let v = iter::repeat_with(|| V::decode(buf))
            .take(len)
            .collect::<anyhow::Result<Vec<V>>>()?;

        Ok(Self(v, PhantomData))
    }
}

impl<L, V> Encoder for LengthPrefixedVec<L, V>
where
    L: Encoder + Decoder + LengthPrefix,
    V: Encoder + Decoder,
{
    fn byte_len(&self) -> usize {
        L::from_len(self.0.len()).byte_len() + self.0.iter().map(|v| v.byte_len()).sum::<usize>()
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        L::from_len(self.0.len()).encode(buf)?;

        for item in &self.0 {
            item.encode(buf)?;
        }

        Ok(())
    }
}

static SEGMENT_BITS: u8 = 0b01111111;
static CONTINUE_BIT: u8 = 0b10000000;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl LengthPrefix for VarInt {
    fn len(&self) -> usize {
        self.0 as usize
    }

    fn from_len(value: usize) -> Self {
        Self(value as i32)
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VarInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Decoder for VarInt {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut byte_buf = [0];

        loop {
            // read a byte from the buffer
            buf.read_exact(&mut byte_buf)?;

            let byte = byte_buf[0];

            value |= ((byte & SEGMENT_BITS) as i32) << position;

            if byte & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                anyhow::bail!("VarInt is too big");
            }
        }

        Ok(Self(value))
    }
}

impl Encoder for VarInt {
    fn byte_len(&self) -> usize {
        for i in 1..5 {
            if (self.0 & -1 << (i * 7)) != 0 {
                continue;
            }

            return i;
        }

        5
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        let mut value = self.0;

        loop {
            if (value & !(SEGMENT_BITS as i32)) == 0 {
                buf.write_all(&[value as u8])?;
                break;
            }

            buf.write_all(&[((value & (SEGMENT_BITS as i32)) | (CONTINUE_BIT as i32)) as u8])?;
            value = ((value as u32) >> 7) as i32;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, PartialOrd, Ord)]
pub struct VarLong(pub i64);

impl LengthPrefix for VarLong {
    fn len(&self) -> usize {
        self.0 as usize
    }

    fn from_len(value: usize) -> Self {
        Self(value as i64)
    }
}

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<VarLong> for i64 {
    fn from(value: VarLong) -> Self {
        value.0
    }
}

impl Deref for VarLong {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VarLong {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Decoder for VarLong {
    fn decode<T: Read>(buf: &mut T) -> anyhow::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut byte_buf = [0];

        loop {
            // read a byte from the buffer
            buf.read_exact(&mut byte_buf)?;

            let byte = byte_buf[0];

            value |= ((byte & SEGMENT_BITS) as i64) << position;

            if byte & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 64 {
                anyhow::bail!("VarLong is too big");
            }
        }

        Ok(Self(value))
    }
}

impl Encoder for VarLong {
    fn byte_len(&self) -> usize {
        for i in 1..10 {
            if (self.0 & -1 << (i * 7)) != 0 {
                continue;
            }

            return i;
        }

        10
    }

    fn encode<T: Write>(&self, buf: &mut T) -> anyhow::Result<()> {
        let mut value = self.0;

        loop {
            if (value & !(SEGMENT_BITS as i64)) == 0 {
                buf.write_all(&[value as u8])?;
                break;
            }

            buf.write_all(&[((value & (SEGMENT_BITS as i64)) | (CONTINUE_BIT as i64)) as u8])?;
            value = ((value as u64) >> 7) as i64;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_varint_decode_single_byte() {
        let mut buf = Cursor::new(vec![0x01]);
        let varint = VarInt::decode(&mut buf).unwrap();
        assert_eq!(varint.0, 1);
    }

    #[test]
    fn test_varint_decode_multi_byte() {
        let mut buf = Cursor::new(vec![0x80, 0x01]);
        let varint = VarInt::decode(&mut buf).unwrap();
        assert_eq!(varint.0, 128);
    }

    #[test]
    fn test_varint_decode_max_positive() {
        let mut buf = Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
        let varint = VarInt::decode(&mut buf).unwrap();
        assert_eq!(varint.0, 2147483647);
    }

    #[test]
    fn test_varint_decode_negative_one() {
        let mut buf = Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        let varint = VarInt::decode(&mut buf).unwrap();
        assert_eq!(varint.0, -1);
    }

    #[test]
    fn test_varint_decode_min_negative() {
        let mut buf = Cursor::new(vec![0x80, 0x80, 0x80, 0x80, 0x08]);
        let varint = VarInt::decode(&mut buf).unwrap();
        assert_eq!(varint.0, -2147483648);
    }

    #[test]
    fn test_varint_decode_too_large() {
        let mut buf = Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]);
        assert!(VarInt::decode(&mut buf).is_err());
    }

    #[test]
    fn test_varlong_decode_single_byte() {
        let mut buf = Cursor::new(vec![0x01]);
        let varlong = VarLong::decode(&mut buf).unwrap();
        assert_eq!(varlong.0, 1);
    }

    #[test]
    fn test_varlong_decode_multi_byte() {
        let mut buf = Cursor::new(vec![0x80, 0x01]);
        let varlong = VarLong::decode(&mut buf).unwrap();
        assert_eq!(varlong.0, 128);
    }

    #[test]
    fn test_varlong_decode_max_positive() {
        let mut buf = Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        let varlong = VarLong::decode(&mut buf).unwrap();
        assert_eq!(varlong.0, 9223372036854775807);
    }

    #[test]
    fn test_varlong_decode_negative_one() {
        let mut buf = Cursor::new(vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
        ]);
        let varlong = VarLong::decode(&mut buf).unwrap();
        assert_eq!(varlong.0, -1);
    }

    #[test]
    fn test_varlong_decode_min_negative() {
        let mut buf = Cursor::new(vec![
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
        ]);
        let varlong = VarLong::decode(&mut buf).unwrap();
        assert_eq!(varlong.0, -9223372036854775808);
    }

    #[test]
    fn test_varlong_decode_too_large() {
        let mut buf = Cursor::new(vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
        ]);
        assert!(VarLong::decode(&mut buf).is_err());
    }

    #[test]
    fn test_varint_encode_single_byte() {
        let varint = VarInt(1);
        let mut buf = Vec::new();
        varint.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x01]);
    }

    #[test]
    fn test_varint_encode_multi_byte() {
        let varint = VarInt(128);
        let mut buf = Vec::new();
        varint.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn test_varint_encode_max_positive() {
        let varint = VarInt(2147483647);
        let mut buf = Vec::new();
        varint.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
    }

    #[test]
    fn test_varint_encode_negative_one() {
        let varint = VarInt(-1);
        let mut buf = Vec::new();
        varint.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn test_varint_encode_min_negative() {
        let varint = VarInt(-2147483648);
        let mut buf = Vec::new();
        varint.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x80, 0x80, 0x80, 0x80, 0x08]);
    }

    #[test]
    fn test_varlong_encode_single_byte() {
        let varlong = VarLong(1);
        let mut buf = Vec::new();
        varlong.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x01]);
    }

    #[test]
    fn test_varlong_encode_multi_byte() {
        let varlong = VarLong(128);
        let mut buf = Vec::new();
        varlong.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn test_varlong_encode_max_positive() {
        let varlong = VarLong(9223372036854775807);
        let mut buf = Vec::new();
        varlong.encode(&mut buf).unwrap();
        assert_eq!(
            buf,
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]
        );
    }

    #[test]
    fn test_varlong_encode_negative_one() {
        let varlong = VarLong(-1);
        let mut buf = Vec::new();
        varlong.encode(&mut buf).unwrap();
        assert_eq!(
            buf,
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
        );
    }

    #[test]
    fn test_varlong_encode_min_negative() {
        let varlong = VarLong(-9223372036854775808);
        let mut buf = Vec::new();
        varlong.encode(&mut buf).unwrap();
        assert_eq!(
            buf,
            vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]
        );
    }
}
