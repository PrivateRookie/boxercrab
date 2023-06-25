use bytes::BytesMut;
use parse_tool::InputBuf;

use crate::codec::{get_null_term_str, Decode, DecodeError, Encode};

use super::{decode_header, Packet};

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_auth_switch_request.html)
#[derive(Debug, Clone)]
pub struct AuthSwitchReq {
    pub plugin_name: String,
    pub plugin_data: BytesMut,
}

impl AuthSwitchReq {
    pub const STATUS: u8 = 254;
}

impl<I: InputBuf> Decode<I> for AuthSwitchReq {
    fn decode(input: &mut I) -> Result<AuthSwitchReq, DecodeError> {
        let tag = input.read_u8_le()?;
        if tag != 0xfe {
            return Err(DecodeError::InvalidData);
        }
        let plugin_name = get_null_term_str(input)?;
        let plugin_data = if input.left() > 0 {
            BytesMut::from_iter(input.read_vec(input.left() - 1)?)
        } else {
            BytesMut::new()
        };
        Ok(AuthSwitchReq {
            plugin_name,
            plugin_data,
        })
    }
}

/// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_auth_switch_response.html
#[derive(Debug, Clone)]
pub struct AuthSwitchResp {
    pub data: BytesMut,
}

impl<I: InputBuf> Decode<I> for Packet<AuthSwitchResp> {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let (len, seq_id) = decode_header(input)?;
        let data = input.read_vec(len.int() as usize)?;
        let data = BytesMut::from_iter(data);
        Ok(Packet {
            len,
            seq_id,
            payload: AuthSwitchResp { data },
        })
    }
}

impl Encode for AuthSwitchResp {
    fn encode(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(&self.data);
    }
}
