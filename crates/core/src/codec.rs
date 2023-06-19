use bytes::{Buf, BufMut};
use bytes::{Bytes, BytesMut};

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

#[derive(Debug, Clone)]
pub struct CheckError;

impl From<CheckError> for DecodeError {
    fn from(_: CheckError) -> Self {
        Self::NoEnoughData
    }
}

macro_rules! impl_check {
    ($check_fn:ident, $raw_fn:ident, $ret:ty, $len:literal) => {
        fn $check_fn(&mut self) -> Result<$ret, CheckError> {
            if self.remaining() >= $len {
                Ok(self.$raw_fn())
            } else {
                Err(CheckError)
            }
        }
    };
    ($check_fn:ident, $raw_fn:ident, $ret:ty ) => {
        fn $check_fn(&mut self, len: usize) -> Result<$ret, CheckError> {
            if self.remaining() >= len {
                Ok(self.$raw_fn(len))
            } else {
                Err(CheckError)
            }
        }
    };
}

pub trait CheckedBuf: Buf + Sized {
    impl_check!(check_u8, get_u8, u8, 1);
    impl_check!(check_i8, get_i8, i8, 1);
    impl_check!(check_u16, get_u16, u16, 2);
    impl_check!(check_u16_le, get_u16_le, u16, 2);
    impl_check!(check_u16_ne, get_u16_ne, u16, 2);
    impl_check!(check_i16, get_i16, i16, 2);
    impl_check!(check_i16_le, get_i16_le, i16, 2);
    impl_check!(check_i16_ne, get_i16_ne, i16, 2);
    impl_check!(check_u32, get_u32, u32, 4);
    impl_check!(check_u32_le, get_u32_le, u32, 4);
    impl_check!(check_u32_ne, get_u32_ne, u32, 4);
    impl_check!(check_i32, get_i32, i32, 4);
    impl_check!(check_i32_le, get_i32_le, i32, 4);
    impl_check!(check_i32_ne, get_i32_ne, i32, 4);
    impl_check!(check_u64, get_u64, u64, 8);
    impl_check!(check_u64_le, get_u64_le, u64, 8);
    impl_check!(check_u64_ne, get_u64_ne, u64, 8);
    impl_check!(check_i64, get_i64, i64, 8);
    impl_check!(check_i64_le, get_i64_le, i64, 8);
    impl_check!(check_i64_ne, get_i64_ne, i64, 8);
    impl_check!(check_u128, get_u128, u128, 16);
    impl_check!(check_u128_le, get_u128_le, u128, 16);
    impl_check!(check_u128_ne, get_u128_ne, u128, 16);
    impl_check!(check_i128, get_i128, i128, 16);
    impl_check!(check_i128_le, get_i128_le, i128, 16);
    impl_check!(check_i128_ne, get_i128_ne, i128, 16);
    impl_check!(check_uint, get_uint, u64);
    impl_check!(check_uint_le, get_uint_le, u64);
    impl_check!(check_uint_ne, get_uint_ne, u64);
    impl_check!(check_int, get_int, i64);
    impl_check!(check_int_le, get_int_le, i64);
    impl_check!(check_int_ne, get_int_ne, i64);

    fn consume(&mut self, len: usize) -> Result<BytesMut, CheckError> {
        if self.remaining() < len {
            return Err(CheckError);
        }
        let mut data = BytesMut::new();
        data.resize(len, 0);
        self.copy_to_slice(&mut data);
        Ok(data)
    }

    /// after this operation, self contain `[at, len)`,
    /// return part contain [0, at)
    fn cut_at(&mut self, at: usize) -> Result<Self, CheckError>;
}

impl CheckedBuf for &[u8] {
    fn cut_at(&mut self, at: usize) -> Result<Self, CheckError> {
        if self.len() < at {
            return Err(CheckError);
        }
        let ret = &self[..at];
        *self = &self[at..];
        Ok(ret)
    }
}

impl CheckedBuf for Bytes {
    fn cut_at(&mut self, at: usize) -> Result<Self, CheckError> {
        if self.len() < at {
            return Err(CheckError);
        }
        Ok(self.split_to(at))
    }
}

impl CheckedBuf for BytesMut {
    fn cut_at(&mut self, at: usize) -> Result<Self, CheckError> {
        if self.len() < at {
            return Err(CheckError);
        }
        Ok(self.split_to(at))
    }
}

pub trait Decode<I: CheckedBuf, Output = Self, Error = DecodeError>: Sized {
    fn decode(input: &mut I) -> Result<Output, Error>;
}
pub type DecodeResult<T> = Result<T, DecodeError>;

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut);
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
                val.0.copy_from_slice(&value.to_le_bytes()[..$idx]);
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

        impl<I: CheckedBuf> Decode<I> for $name {
            fn decode(input: &mut I) -> Result<Self, DecodeError> {
                if input.remaining() < $len {
                    return Err(DecodeError::NoEnoughData);
                };
                let mut buf = [0u8; $len];
                input.copy_to_slice(&mut buf);
                Ok(Self(buf))
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

impl<I: CheckedBuf> Decode<I> for VLenInt {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        match input.check_u8()? {
            val @ 0..=0xfb => Ok(Self(val as u64)),
            0xfc => Ok(Self(input.check_u16_le()? as u64)),
            0xfd => {
                let i = Int3::decode(input)?;
                Ok(Self(i.int() as u64))
            }
            0xfe => Ok(Self(input.check_u64_le()?)),
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

pub fn get_null_term_bytes<I: CheckedBuf>(input: &mut I) -> Result<I, DecodeError> {
    let pos = input
        .chunk()
        .iter()
        .position(|b| *b == b'\0')
        .ok_or(DecodeError::MissingNull)?;
    let data = input.cut_at(pos)?;
    input.consume(1)?;
    Ok(data)
}

pub fn put_null_term_bytes(input: impl AsRef<[u8]>, buf: &mut BytesMut) {
    buf.extend_from_slice(input.as_ref());
    buf.put_u8(b'\0');
}

pub fn get_var_bytes<I: CheckedBuf>(input: &mut I) -> Result<I, DecodeError> {
    let len = VLenInt::decode(input)?.0 as usize;
    let data = input.cut_at(len)?;
    Ok(data)
}

pub fn put_var_bytes(input: impl AsRef<[u8]>, buf: &mut BytesMut) {
    let len = input.as_ref().len() as u64;
    let len = VLenInt::new(len);
    len.encode(buf);
    buf.extend_from_slice(input.as_ref());
}

pub fn get_null_term_str<I: CheckedBuf>(input: &mut I) -> Result<String, DecodeError> {
    let raw = get_null_term_bytes(input)?;
    String::from_utf8(raw.chunk().to_vec()).map_err(|_| DecodeError::InvalidUtf8)
}

pub fn put_null_term_str(s: &str, buf: &mut BytesMut) {
    put_null_term_bytes(s.as_bytes(), buf)
}

pub fn get_var_str<I: CheckedBuf>(input: &mut I) -> Result<String, DecodeError> {
    let raw = get_var_bytes(input)?;
    String::from_utf8(raw.chunk().to_vec()).map_err(|_| DecodeError::InvalidUtf8)
}

pub fn put_var_str(s: &str, buf: &mut BytesMut) {
    let data = s.as_bytes();
    put_var_bytes(data, buf)
}
