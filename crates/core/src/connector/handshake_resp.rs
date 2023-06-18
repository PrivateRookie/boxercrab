use std::collections::HashMap;

use bytes::BytesMut;

use crate::{
    connector::{consume, fix_bytes, null_term_string, var_bytes, var_string},
    data::{FixInt1, FixInt3, FixInt4},
    parser::Decode,
};

use super::Capabilities;

#[derive(Debug, Clone)]
pub struct HandshakeResponse41 {
    pub caps: Capabilities,
    pub max_packet_size: FixInt3,
    pub charset: FixInt1,
    pub user_name: String,
    pub auth_resp: BytesMut,
    pub database: Option<String>,
    pub plugin_name: Option<String>,
    pub connect_attrs: HashMap<String, String>,
    pub zstd_level: FixInt1,
}

impl Decode for HandshakeResponse41 {
    fn parse(buf: &mut BytesMut) -> Result<Self, crate::parser::ParseError> {
        let caps = FixInt4::parse(buf)?;
        let caps = Capabilities::from_bits(caps.int()).unwrap();
        let max_packet_size = FixInt3::parse(buf)?;
        let charset = FixInt1::parse(buf)?;
        consume(buf, 23)?;
        let user_name = null_term_string(buf)?;
        let auth_resp = if caps.contains(Capabilities::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA) {
            var_bytes(buf)?
        } else {
            let len = FixInt1::parse(buf)?.int() as usize;
            fix_bytes(buf, len)?
        };
        let database = if caps.contains(Capabilities::CLIENT_CONNECT_WITH_DB) {
            Some(null_term_string(buf)?)
        } else {
            None
        };
        let plugin_name = if caps.contains(Capabilities::CLIENT_PLUGIN_AUTH) {
            Some(null_term_string(buf)?)
        } else {
            None
        };
        let mut connect_attrs: HashMap<String, String> = Default::default();
        if caps.contains(Capabilities::CLIENT_CONNECT_ATTRS) {
            let count = FixInt1::parse(buf)?.int();
            for _ in 0..count {
                let key = var_string(buf)?;
                let val = var_string(buf)?;
                connect_attrs.insert(key, val);
            }
        }
        let zstd_level = FixInt1::parse(buf)?;
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
