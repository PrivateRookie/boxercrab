use crate::codec::{
    CheckedBuf, Decode, DecodeError, DecodeResult, Encode, Int1, Int2, Int3, VLenInt,
};

mod handshake_v10;
use bytes::BytesMut;
pub use handshake_v10::*;
mod handshake_resp;
pub use handshake_resp::*;
mod auth;
pub use auth::*;

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_basic_packets.html#sect_protocol_basic_packets_packet)
#[derive(Debug, Clone)]
pub struct Packet<P> {
    pub len: Int3,
    pub seq_id: Int1,
    pub payload: P,
}

pub fn decode_header<I: CheckedBuf>(input: &mut I) -> DecodeResult<(Int3, Int1)> {
    let len = Int3::decode(input)?;
    let seq_id = Int1::decode(input)?;
    if input.remaining() < len.int() as usize {
        return Err(DecodeError::NoEnoughData);
    }
    Ok((len, seq_id))
}

pub fn decode_packet<I: CheckedBuf, P: Decode<I>>(input: &mut I) -> DecodeResult<Packet<P>> {
    let (len, seq_id) = decode_header(input)?;
    let payload = P::decode(input)?;
    Ok(Packet {
        len,
        seq_id,
        payload,
    })
}

pub fn encode_packet<P: Encode>(seq_id: u8, payload: &P, buf: &mut BytesMut) {
    let start = buf.len();
    buf.extend_from_slice(&[0, 0, 0]);
    Int1::from(seq_id).encode(buf);
    payload.encode(buf);
    let end = buf.len();
    let len = end - start;
    buf[start..(start + 3)].copy_from_slice(Int3::from(len as u32).bytes())
}

#[allow(unused_macros)]
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
