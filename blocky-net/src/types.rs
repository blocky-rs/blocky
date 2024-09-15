use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

use crate::{decoder::Decoder, encoder::Encoder};

static SEGMENT_BITS: u8 = 0b01111111;
static CONTINUE_BIT: u8 = 0b10000000;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
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
            if (self.0 & -1 << i * 7) != 0 {
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
                buf.write(&[value as u8])?;
                break;
            }

            buf.write(&[((value & (SEGMENT_BITS as i32)) | (CONTINUE_BIT as i32)) as u8])?;
            value = ((value as u32) >> 7) as i32;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, PartialOrd, Ord)]
pub struct VarLong(pub i64);

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        Self(value)
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
            if (self.0 & -1 << i * 7) != 0 {
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
                buf.write(&[value as u8])?;
                break;
            }

            buf.write(&[((value & (SEGMENT_BITS as i64)) | (CONTINUE_BIT as i64)) as u8])?;
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
