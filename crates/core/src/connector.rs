use bytes::{Buf, BytesMut};
use sha1::digest::block_buffer::Error;

use crate::{
    data::{Int1, Int2, Int3, VLenInt},
    parser::{Decode, ParseError},
};

mod handshake_v10;
pub use handshake_v10::*;
mod handshake_resp;
pub use handshake_resp::*;
mod auth;
pub use auth::*;

#[derive(Debug, Clone)]
pub struct Packet<P> {
    pub len: Int3,
    pub seq_id: Int1,
    pub payload: P,
}

#[derive(Debug, Clone)]
pub struct CheckError;

impl From<CheckError> for ParseError {
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

pub trait CheckedBuf: Buf {
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
}

impl<T: Buf> CheckedBuf for T {}

pub trait De<I: CheckedBuf, Output, Ctx = (), Error = ParseError>: Sized {
    fn der(ctx: Ctx, input: I) -> Result<Output, Error>;
}

impl<P: Decode> Decode for Packet<P> {
    fn parse(buf: &mut BytesMut) -> Result<Self, ParseError> {
        let len = Int3::parse(buf)?;
        let seq_id = Int1::parse(buf)?;
        let mut payload_data = buf.split_to(len.int() as usize);
        let payload = P::parse(&mut payload_data)?;
        assert!(payload_data.is_empty());
        Ok(Self {
            len,
            seq_id,
            payload,
        })
    }
}

fn null_term_string(buf: &mut BytesMut) -> Result<String, ParseError> {
    let raw = &null_term_bytes(buf)?;
    String::from_utf8(raw.to_vec()).map_err(|_| ParseError::InvalidUtf8)
}

fn null_term_bytes(buf: &mut BytesMut) -> Result<BytesMut, ParseError> {
    let pos = buf
        .iter()
        .position(|b| *b == b'\0')
        .ok_or(ParseError::MissingNull)?;
    let bytes = buf.split_to(pos);
    buf.advance(1);
    Ok(bytes)
}

fn fix_bytes(buf: &mut BytesMut, len: usize) -> Result<BytesMut, ParseError> {
    if buf.len() < len {
        return Err(ParseError::NoEnoughData);
    }
    Ok(buf.split_to(len))
}

fn var_bytes(buf: &mut BytesMut) -> Result<BytesMut, ParseError> {
    let len = VLenInt::parse(buf)?.0 as usize;
    fix_bytes(buf, len)
}

fn consume(buf: &mut BytesMut, len: usize) -> Result<(), ParseError> {
    fix_bytes(buf, len)?;
    Ok(())
}

fn fix_string(buf: &mut BytesMut, len: usize) -> Result<String, ParseError> {
    let raw = fix_bytes(buf, len)?.to_vec();
    String::from_utf8(raw).map_err(|_| ParseError::InvalidUtf8)
}

fn var_string(buf: &mut BytesMut) -> Result<String, ParseError> {
    let len = VLenInt::parse(buf)?.0 as usize;
    fix_string(buf, len)
}

macro_rules! hex {
    ($data:literal) => {{
        let buf = bytes::BytesMut::from_iter(
            (0..$data.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&$data[i..i + 2], 16).unwrap()),
        );
        buf
    }};
}

#[test]
fn data() {
    macro_rules! sha1 {
        ($($d:expr),*) => {{
            let mut hasher = Sha1::new();
            $(hasher.update($d);)*
            let i: [u8; 20] = hasher.finalize().into();
            i
        }};
    }
    fn hash(data: &[u8]) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    use sha1::{Digest, Sha1};
    let s = hex!("3d2e135a3f3b6e7c4e08604316667e517e746e28");
    let mut h1 = sha1!(b"1234");
    let h2 = hash(&h1);
    let multi = sha1!(&s, h2);
    for i in 0..20 {
        h1[i] ^= multi[i];
    }
    println!("{:?}", h1);
    let c = hex!("a2c5096aeadc27fac151d6d6ec428becd09358f4");
    println!("{:?}", c.to_vec());
}

#[derive(Debug, Clone)]
pub struct OkPacket {
    pub header: Int1,
    pub affected_rows: VLenInt,
    pub last_insert_id: VLenInt,
    pub status_flags: Int2,
    pub warnings: Int2,
    pub info: String,
}

#[derive(Debug, Clone)]
pub struct ErrPacket {
    pub header: Int1,
    pub code: Int2,
    pub sql_state_marker: Int1,
    pub sql_state: String,
    pub error_msg: String,
}
