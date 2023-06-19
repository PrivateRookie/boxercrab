use bytes::BytesMut;

use crate::codec::{get_null_term_str, CheckedBuf, Decode, DecodeError};

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

impl<I: CheckedBuf> Decode<I> for Packet<AuthSwitchReq> {
    fn decode(input: &mut I) -> Result<Packet<AuthSwitchReq>, DecodeError> {
        let (len, seq_id) = decode_header(input)?;
        let r1 = input.remaining();
        let tag = input.check_u8()?;
        if tag != 0xfe {
            return Err(DecodeError::InvalidData);
        }
        let plugin_name = get_null_term_str(input)?;
        let r2 = input.remaining();
        let remain = len.int() as usize - (r1 - r2);
        let plugin_data = BytesMut::from_iter(input.cut_at(remain)?.chunk());
        Ok(Packet {
            len,
            seq_id,
            payload: AuthSwitchReq {
                plugin_name,
                plugin_data,
            },
        })
    }
}

/// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_auth_switch_response.html
#[derive(Debug, Clone)]
pub struct AuthSwitchResp {
    pub data: BytesMut,
}

impl<I: CheckedBuf> Decode<I> for Packet<AuthSwitchResp> {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let (len, seq_id) = decode_header(input)?;
        let data = input.consume(len.int() as usize)?;
        Ok(Packet {
            len,
            seq_id,
            payload: AuthSwitchResp { data },
        })
    }
}
