use bytes::{Buf, BytesMut};
use sha1::digest::block_buffer::Error;

use crate::{
    data::{FixInt1, FixInt2, FixInt3, VLenInt},
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
    pub len: FixInt3,
    pub seq_id: FixInt1,
    pub payload: P,
}

pub struct S8(i8);

pub struct S16(i16);

pub struct CheckCtx;
pub struct EmitCtx;

pub trait De<Ctx, Input, Output, Error>: Sized {
    fn go(ctx: Ctx, input: Input) -> Result<(Input, Output), Error>;
}

impl<P: Decode> Decode for Packet<P> {
    fn parse(buf: &mut BytesMut) -> Result<Self, ParseError> {
        let len = FixInt3::parse(buf)?;
        let seq_id = FixInt1::parse(buf)?;
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
        return Err(ParseError::NotEnoughData {
            expected: len,
            got: buf.len(),
        });
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
    pub header: FixInt1,
    pub affected_rows: VLenInt,
    pub last_insert_id: VLenInt,
    pub status_flags: FixInt2,
    pub warnings: FixInt2,
    pub info: String,
}

#[derive(Debug, Clone)]
pub struct ErrPacket {
    pub header: FixInt1,
    pub code: FixInt2,
    pub sql_state_marker: FixInt1,
    pub sql_state: String,
    pub error_msg: String,
}
