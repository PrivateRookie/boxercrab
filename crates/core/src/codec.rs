use bytes::BufMut;
use bytes::BytesMut;

use parse_tool::{CheckError, InputBuf};
use thiserror::Error;
#[derive(Debug, Clone, Error)]
pub enum DecodeError {
    #[error("no enough data")]
    NoEnoughData,
    #[error("invalid utf-8 string")]
    InvalidUtf8,
    #[error("missing terminal null bytes")]
    MissingNull,
    #[error("invalid data")]
    InvalidData,
}

impl From<CheckError> for DecodeError {
    fn from(_: CheckError) -> Self {
        Self::NoEnoughData
    }
}

pub trait Decode<I: InputBuf, Output = Self, Error = DecodeError>: Sized {
    fn decode(input: &mut I) -> Result<Output, Error>;
}
pub type DecodeResult<T> = Result<T, DecodeError>;

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

impl<I: InputBuf> Decode<I> for Vec<u8> {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        Ok(input.read_to_end())
    }
}

macro_rules! from_prime {
    ($t:ty, $name:ident) => {
        impl From<$t> for $name {
            fn from(value: $t) -> Self {
                Self(value.to_le_bytes())
            }
        }
    };
    ($t:ty, $name:ident, $max:expr, $idx:expr) => {
        impl From<$t> for $name {
            fn from(value: $t) -> Self {
                assert!(value <= $max);
                let mut val = Self::default();
                val.0.copy_from_slice(&value.to_le_bytes()[..($idx + 1)]);
                val
            }
        }
    };
}

macro_rules! custom_impl {
    ($t:ty, $name:ident, $len:literal) => {
        impl $name {
            pub fn new(value: [u8; $len]) -> $name {
                Self(value)
            }

            pub fn int(&self) -> $t {
                let data: $t = 0;
                let mut data = data.to_le_bytes();
                let len = self.0.len();
                data[..len].copy_from_slice(&self.0);
                <$t>::from_le_bytes(data)
            }

            pub fn bytes(&self) -> &[u8] {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.int())
            }
        }

        impl<I: InputBuf> Decode<I> for $name {
            fn decode(input: &mut I) -> Result<Self, DecodeError> {
                Ok(Self(input.read_array()?))
            }
        }

        impl Encode for $name {
            fn encode(&self, buf: &mut BytesMut) {
                buf.extend_from_slice(&self.0);
            }
        }
    };
}

macro_rules! fix {
    ($name:ident, $len:literal, $min_ty:ty, $max:expr) => {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name(pub(crate) [u8; $len]);

        from_prime!($min_ty, $name, $max, $len - 1);
        custom_impl!($min_ty, $name, $len);
    };
    ($name:ident, $len:literal, $min_ty:ty) => {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name(pub(crate) [u8; $len]);
        from_prime!($min_ty, $name);
        custom_impl!($min_ty, $name, $len);
    };
}

fix!(Int1, 1, u8);
fix!(Int2, 2, u16);
fix!(Int3, 3, u32, u32::MAX >> 1);
fix!(Int4, 4, u32);
fix!(Int6, 6, u64, u64::MAX >> 2);
fix!(Int8, 8, u64);

/// variable length int
#[derive(Default, Debug, Clone, Copy)]
pub struct VLenInt(pub u64);

impl VLenInt {
    pub fn new(val: u64) -> Self {
        Self(val)
    }

    pub fn int(&self) -> u64 {
        self.0
    }

    pub fn bytes(&self) -> BytesMut {
        let mut data = BytesMut::new();
        self.encode(&mut data);
        data
    }
}

impl<I: InputBuf> Decode<I> for VLenInt {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        match input.read_u8_le()? {
            val @ 0..=0xfb => Ok(Self(val as u64)),
            0xfc => Ok(Self(input.read_u16_le()? as u64)),
            0xfd => {
                let i = Int3::decode(input)?;
                Ok(Self(i.int() as u64))
            }
            0xfe => Ok(Self(input.read_u64_le()?)),
            0xff => Err(DecodeError::InvalidData),
        }
    }
}

impl Encode for VLenInt {
    fn encode(&self, buf: &mut BytesMut) {
        match self.0 {
            0..=250 => buf.put_u8(self.0 as u8),
            251..=65535 => {
                buf.put_u8(0xfc);
                buf.extend_from_slice(&(self.0 as u16).to_le_bytes());
            }
            65536..=16777215 => {
                buf.put_u8(0xfd);
                buf.extend_from_slice(&(self.0 as u32).to_le_bytes()[..2]);
            }
            16777216.. => {
                buf.put_u8(0xfe);
                buf.extend_from_slice(&self.0.to_le_bytes());
            }
        }
    }
}

pub fn get_null_term_bytes<I: InputBuf>(input: &mut I) -> Result<Vec<u8>, DecodeError> {
    let pos = input
        .slice()
        .iter()
        .position(|b| *b == b'\0')
        .ok_or(DecodeError::MissingNull)?;
    let data = input.read_vec(pos)?;
    input.jump_to(1)?;
    Ok(data)
}

pub fn put_null_term_bytes(input: impl AsRef<[u8]>, buf: &mut BytesMut) {
    buf.extend_from_slice(input.as_ref());
    buf.put_u8(b'\0');
}

pub fn get_var_bytes<I: InputBuf>(input: &mut I) -> Result<Vec<u8>, DecodeError> {
    let len = VLenInt::decode(input)?.0 as usize;
    let data = input.read_vec(len)?;
    Ok(data)
}

pub fn put_var_bytes(input: impl AsRef<[u8]>, buf: &mut BytesMut) {
    let len = input.as_ref().len() as u64;
    let len = VLenInt::new(len);
    len.encode(buf);
    buf.extend_from_slice(input.as_ref());
}

pub fn get_null_term_str<I: InputBuf>(input: &mut I) -> Result<String, DecodeError> {
    let raw = get_null_term_bytes(input)?;
    String::from_utf8(raw).map_err(|_| DecodeError::InvalidUtf8)
}

pub fn put_null_term_str(s: &str, buf: &mut BytesMut) {
    put_null_term_bytes(s.as_bytes(), buf)
}

pub fn get_var_str<I: InputBuf>(input: &mut I) -> Result<String, DecodeError> {
    let raw = get_var_bytes(input)?;
    String::from_utf8(raw).map_err(|_| DecodeError::InvalidUtf8)
}

pub fn put_var_str(s: &str, buf: &mut BytesMut) {
    let data = s.as_bytes();
    put_var_bytes(data, buf)
}
