use crate::codec::{
    get_var_bytes, get_var_str, Decode, DecodeError, DecodeResult, Encode, Int1, Int2, Int3, Int4,
    VLenInt,
};

mod handshake_v10;
use bytes::{BufMut, BytesMut};
pub use handshake_v10::*;
mod handshake_resp;
pub use handshake_resp::*;
mod auth;
pub use auth::*;
use parse_tool::InputBuf;

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_basic_packets.html#sect_protocol_basic_packets_packet)
#[derive(Debug, Clone)]
pub struct Packet<P> {
    pub len: Int3,
    pub seq_id: Int1,
    pub payload: P,
}

pub fn decode_header<I: InputBuf>(input: &mut I) -> DecodeResult<(Int3, Int1)> {
    let len = Int3::decode(input)?;
    let seq_id = Int1::decode(input)?;
    if input.left() < len.int() as usize {
        return Err(DecodeError::NoEnoughData);
    }
    Ok((len, seq_id))
}

pub fn decode_packet<I: InputBuf, P: Decode<I>>(input: &mut I) -> DecodeResult<Packet<P>> {
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
    let len = end - start - 4;
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

macro_rules! sha1 {
    ($($d:expr),*) => {{
        let mut hasher = Sha1::new();
        $(hasher.update($d);)*
        let i: [u8; 20] = hasher.finalize().into();
        i
    }};
}

pub fn native_password_auth(password: &[u8], auth_data: &[u8]) -> [u8; 20] {
    use sha1::{Digest, Sha1};
    let mut h1 = sha1!(password);
    let h2 = sha1!(&h1);
    let multi = sha1!(&auth_data, h2);
    for i in 0..20 {
        h1[i] ^= multi[i];
    }
    h1
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

impl OkPacket {
    pub fn is_ok(&self) -> bool {
        self.header.int() == 0x00
    }

    pub fn is_eof(&self) -> bool {
        self.header.int() == 0xfe
    }
}

impl<I: InputBuf> Decode<I> for OkPacket {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let header = Int1::decode(input)?;
        let affected_rows = VLenInt::decode(input)?;
        let last_insert_id = VLenInt::decode(input)?;
        let status_flags = Int2::decode(input)?;
        let warnings = Int2::decode(input)?;
        let info = String::new();
        Ok(Self {
            header,
            affected_rows,
            last_insert_id,
            status_flags,
            warnings,
            info,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ErrPacket {
    pub header: Int1,
    pub code: Int2,
    pub sql_state_marker: Int1,
    pub sql_state: String,
    pub error_msg: String,
}

impl<I: InputBuf> Decode<I> for ErrPacket {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let header = Int1::decode(input)?;
        let code = Int2::decode(input)?;
        let sql_state_marker = Int1::decode(input)?;
        let sql_state =
            String::from_utf8(input.read_vec(5)?).map_err(|_| DecodeError::InvalidUtf8)?;
        let error_msg =
            String::from_utf8(input.read_to_end()).map_err(|_| DecodeError::InvalidUtf8)?;
        Ok(Self {
            header,
            code,
            sql_state,
            sql_state_marker,
            error_msg,
        })
    }
}

#[derive(Debug, Clone)]
pub enum OkOrErr {
    Ok(OkPacket),
    Err(ErrPacket),
}

impl<I: InputBuf> Decode<I> for OkOrErr {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        match input.slice()[0] {
            0xff => ErrPacket::decode(input).map(Self::Err),
            0xfe | 0x0 => OkPacket::decode(input).map(Self::Ok),
            _ => Err(DecodeError::InvalidData),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComQuit;

impl Encode for ComQuit {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(0x01);
    }
}

/// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_com_query.html
#[derive(Debug, Clone)]
pub struct ComQuery {
    pub query: String,
}

impl Encode for ComQuery {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(0x03);
        buf.extend_from_slice(self.query.as_bytes());
    }
}

impl<T: ToString> From<T> for ComQuery {
    fn from(value: T) -> Self {
        Self {
            query: value.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextResultSet {
    pub column_count: VLenInt,
    pub col_defs: Vec<ColDef>,
    pub rows: Vec<TextResult>
}

/// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_com_query_response.html
#[derive(Debug, Clone)]
pub struct TextResult {
    pub columns: Vec<Vec<u8>>,
}

impl<I: InputBuf> Decode<I> for TextResult {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let mut columns = vec![];
        while input.left() > 0 {
            let col = get_var_bytes(input)?;
            columns.push(col);
        }
        Ok(Self { columns })
    }
}

/// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_com_query_response_text_resultset_column_definition.html
#[derive(Debug, Clone)]
pub struct ColDef {
    pub catalog: String,
    pub schema: String,
    pub table: String,
    pub original_table: String,
    pub name: String,
    pub original_name: String,
    pub length_of_fixed_length_fields: VLenInt,
    pub charset: Int2,
    pub column_length: Int4,
    pub ty: Int1,
    pub flags: Int2,
    pub decimals: Int1,
}

impl<I: InputBuf> Decode<I> for ColDef {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let catalog = get_var_str(input)?;
        let schema = get_var_str(input)?;
        let table = get_var_str(input)?;
        let original_table = get_var_str(input)?;
        let name = get_var_str(input)?;
        let original_name = get_var_str(input)?;
        let length_of_fixed_length_fields = VLenInt::decode(input)?;
        let charset = Int2::decode(input)?;
        let column_length = Int4::decode(input)?;
        let ty = Int1::decode(input)?;
        let flags = Int2::decode(input)?;
        let decimals = Int1::decode(input)?;
        Ok(Self {
            catalog,
            schema,
            table,
            original_table,
            name,
            original_name,
            length_of_fixed_length_fields,
            charset,
            column_length,
            ty,
            flags,
            decimals,
        })
    }
}

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_com_binlog_dump.html)
#[derive(Debug, Clone)]
pub struct ComBinLogDump {
    pub pos: Int4,
    pub flags: Int2,
    pub server_id: Int4,
    pub filename: String,
}

impl Encode for ComBinLogDump {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(0x12);
        self.pos.encode(buf);
        self.flags.encode(buf);
        self.server_id.encode(buf);
        buf.extend_from_slice(self.filename.as_bytes());
    }
}
