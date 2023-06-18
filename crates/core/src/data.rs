use bytes::{Buf, BufMut, BytesMut};

use crate::{
    connector::De,
    parser::{Decode, ParseError},
};

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

        impl Decode for $name {
            #[allow(clippy::len_zero)]
            fn parse(buf: &mut BytesMut) -> Result<Self, ParseError> {
                let len = buf.len();
                if buf.len() < $len {
                    return Err(ParseError::NotEnoughData {
                        expected: 1,
                        got: len,
                    });
                }
                let mut data = [0u8; $len];
                data.copy_from_slice(&buf.split_to($len));
                Ok(Self::new(data))
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

fix!(FixInt1, 1, u8);
fix!(FixInt2, 2, u16);
fix!(FixInt3, 3, u32, u32::MAX >> 1);
fix!(FixInt4, 4, u32);
fix!(FixInt6, 6, u64, u64::MAX >> 2);
fix!(FixInt8, 8, u64);

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
        match self.0 {
            0..=250 => data.put_u8(self.0 as u8),
            251..=65535 => {
                data.put_u8(0xfc);
                data.extend_from_slice(&(self.0 as u16).to_le_bytes());
            }
            65536..=16777215 => {
                data.put_u8(0xfd);
                data.extend_from_slice(&(self.0 as u32).to_le_bytes()[..2]);
            }
            16777216.. => {
                data.put_u8(0xfe);
                data.extend_from_slice(&self.0.to_le_bytes());
            }
        }
        data
    }
}

impl De<(), BytesMut, Self, ParseError> for VLenInt {
    fn go(ctx: (), input: BytesMut) -> Result<(BytesMut, Self), ParseError> {
        todo!()
    }
}

impl Decode for VLenInt {
    fn parse(buf: &mut BytesMut) -> Result<Self, ParseError> {
        if buf.is_empty() {
            return Err(ParseError::NotEnoughData {
                expected: 1,
                got: 0,
            });
        }
        match buf.get_u8() {
            val @ 0..=0xfb => Ok(Self(val as u64)),
            0xfc => Ok(Self(buf.get_u16_le() as u64)),
            0xfd => {
                let i = FixInt3::parse(buf)?;
                Ok(Self(i.int() as u64))
            }
            0xfe => Ok(Self(buf.get_u64_le())),
            0xff => Err(ParseError::InvalidData),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NullTermString(pub String);

#[derive(Debug, Clone)]
pub struct FixString(pub String);

pub struct VarString(pub String);

#[derive(Debug, Clone)]
pub struct NullTermBytes(pub BytesMut);

#[derive(Debug, Clone)]
pub struct FixBytes(pub BytesMut);

pub struct VarBytes(pub BytesMut);
