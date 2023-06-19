use std::collections::HashMap;

use bytes::BytesMut;

use crate::codec::{
    get_null_term_str, get_var_bytes, get_var_str, put_null_term_str, put_var_bytes, put_var_str,
    CheckedBuf, Decode, DecodeError, Encode, Int1, Int3, Int4, VLenInt,
};

use super::Capabilities;

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_handshake_response.html#sect_protocol_connection_phase_packets_protocol_handshake_response41)
#[derive(Debug, Clone)]
pub struct HandshakeResponse41 {
    pub caps: Capabilities,
    pub max_packet_size: Int4,
    pub charset: Int1,
    pub user_name: String,
    pub auth_resp: BytesMut,
    pub database: Option<String>,
    pub plugin_name: Option<String>,
    pub connect_attrs: HashMap<String, String>,
    pub zstd_level: Int1,
}

impl<I: CheckedBuf> Decode<I> for HandshakeResponse41 {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let caps = Int4::decode(input)?;
        let caps = Capabilities::from_bits(caps.int()).unwrap();
        let max_packet_size = Int4::decode(input)?;
        let charset = Int1::decode(input)?;
        input.consume(23)?;
        let user_name = get_null_term_str(input)?;
        let auth_resp = if caps.contains(Capabilities::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA) {
            get_var_bytes(input)?
        } else {
            let len = Int1::decode(input)?.int() as usize;
            input.cut_at(len)?
        };
        let auth_resp = BytesMut::from_iter(auth_resp.chunk());
        let database = if caps.contains(Capabilities::CLIENT_CONNECT_WITH_DB) {
            Some(get_null_term_str(input)?)
        } else {
            None
        };
        let plugin_name = if caps.contains(Capabilities::CLIENT_PLUGIN_AUTH) {
            Some(get_null_term_str(input)?)
        } else {
            None
        };
        let mut connect_attrs: HashMap<String, String> = Default::default();
        if caps.contains(Capabilities::CLIENT_CONNECT_ATTRS) {
            let count = Int1::decode(input)?.int();
            for _ in 0..count {
                let key = get_var_str(input)?;
                let val = get_var_str(input)?;
                connect_attrs.insert(key, val);
            }
        }
        let zstd_level = Int1::decode(input)?;
        Ok(Self {
            caps,
            max_packet_size,
            charset,
            user_name,
            auth_resp,
            database,
            plugin_name,
            connect_attrs,
            zstd_level,
        })
    }
}

impl Encode for HandshakeResponse41 {
    fn encode(&self, buf: &mut BytesMut) {
        Int4::from(self.caps.bits()).encode(buf);
        self.max_packet_size.encode(buf);
        self.charset.encode(buf);
        buf.extend_from_slice(&vec![0].repeat(23));
        put_null_term_str(&self.user_name, buf);
        if self
            .caps
            .contains(Capabilities::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA)
        {
            put_var_bytes(&self.auth_resp, buf)
        } else {
            let len = Int1::from(self.auth_resp.len() as u8);
            len.encode(buf);
            buf.extend_from_slice(&self.auth_resp);
        }
        if self.caps.contains(Capabilities::CLIENT_CONNECT_WITH_DB) {
            put_null_term_str(self.database.as_deref().unwrap_or("default"), buf);
        }
        if self.caps.contains(Capabilities::CLIENT_PLUGIN_AUTH) {
            put_null_term_str(self.plugin_name.as_deref().unwrap_or_default(), buf);
        }
        if self.caps.contains(Capabilities::CLIENT_CONNECT_ATTRS) {
            let len = VLenInt::new(self.connect_attrs.len() as u64);
            len.encode(buf);
            for (k, v) in self.connect_attrs.iter() {
                put_var_str(k, buf);
                put_var_str(v, buf);
            }
        }
        self.zstd_level.encode(buf);
    }
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
fn d() {
    let mut data = hex!("020000004500006830124000800600007f0000017f000001eafc0cea50b960729e57a7da501820fa13b300004000000101022800000001ff00000000000000000000000000000000000000000000007465737400006d7973716c5f6e61746976655f70617373776f72640000");
    dbg!(HandshakeResponse41::decode(&mut data));
}
