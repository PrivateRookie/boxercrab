use bytes::BytesMut;


/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_auth_switch_request.html)
#[derive(Debug, Clone)]
pub struct AuthSwitchReq {
    pub plugin_name: String,
    pub plugin_data: BytesMut,
}

impl AuthSwitchReq {
    pub const STATUS: u8 = 254;
}

/// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_connection_phase_packets_protocol_auth_switch_response.html
#[derive(Debug, Clone)]
pub struct AuthSwitchResp {
    pub data: BytesMut,
}
