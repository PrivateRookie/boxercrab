use super::{consume, fix_bytes, null_term_string};
use crate::data::{FixInt1, FixInt2, FixInt4};
use crate::parser::Decode;
use bytes::BytesMut;

#[derive(Debug, Clone)]
pub struct HandshakeV10 {
    pub protocol_version: FixInt1,
    pub server_version: String,
    pub thread_id: FixInt4,
    pub caps: Capabilities,
    /// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_basic_character_set.html#a_protocol_character_set)
    pub charset: FixInt1,
    pub status: FixInt2,
    pub auth_plugin_name: String,
    pub auth_plugin_data: BytesMut,
}

impl Decode for HandshakeV10 {
    fn parse(buf: &mut BytesMut) -> Result<Self, crate::parser::ParseError> {
        let protocol_version = FixInt1::parse(buf)?;
        if protocol_version.int() != 10 {
            return Err(crate::parser::ParseError::InvalidData);
        }
        let server_version = null_term_string(buf)?;
        let thread_id = FixInt4::parse(buf)?;
        let mut auth_plugin_data = fix_bytes(buf, 8)?;
        consume(buf, 1)?;
        let l_cap = FixInt2::parse(buf)?;
        let charset = FixInt1::parse(buf)?;
        let status = FixInt2::parse(buf)?;
        let h_cap = FixInt2::parse(buf)?;

        let mut caps = [0u8; 4];
        caps[..2].copy_from_slice(l_cap.bytes());
        caps[2..].copy_from_slice(h_cap.bytes());
        let caps = Capabilities::from_bits(u32::from_le_bytes(caps)).unwrap();
        let auth_data_len = FixInt1::parse(buf)?.int();
        consume(buf, 10)?;
        if auth_data_len > 0 {
            dbg!(auth_data_len);
            let len = 13.max(auth_data_len - 8) as usize;
            auth_plugin_data.extend_from_slice(&fix_bytes(buf, len)?);
        }
        let auth_plugin_name = if caps.contains(Capabilities::CLIENT_PLUGIN_AUTH) {
            null_term_string(buf)?
        } else {
            Default::default()
        };
        Ok(Self {
            protocol_version,
            server_version,
            thread_id,
            caps,
            charset,
            status,
            auth_plugin_name,
            auth_plugin_data,
        })
    }
}

bitflags::bitflags! {
    /// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/group__group__cs__capabilities__flags.html#ga07344a4eb8f5c74ea8875bb4e9852fb0)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Capabilities: u32 {
        /// Use the improved version of Old Password Authentication. More...
        const  	CLIENT_LONG_PASSWORD =  1;

        /// Send found rows instead of affected rows in EOF_Packet. More...
        const  	CLIENT_FOUND_ROWS =  2;

        /// Get all column flags. More...
        const  	CLIENT_LONG_FLAG =  4;

        /// Database (schema) name can be specified on connect in Handshake Response Packet. More...
        const  	CLIENT_CONNECT_WITH_DB =  8;

        /// DEPRECATED: Don't allow database.table.column. More...
        const  	CLIENT_NO_SCHEMA =   16;

        /// Compression protocol supported. More...
        const  	CLIENT_COMPRESS =  32;

        /// Special handling of ODBC behavior. More...
        const  	CLIENT_ODBC =  64;

        /// Can use LOAD DATA LOCAL. More...
        const  	CLIENT_LOCAL_FILES =  128;

        /// Ignore spaces before '('. More...
        const  	CLIENT_IGNORE_SPACE =  256;

        /// New 4.1 protocol. More...
        const  	CLIENT_PROTOCOL_41 =  512;

        /// This is an interactive client. More...
        const  	CLIENT_INTERACTIVE =  1024;

        /// Use SSL encryption for the session. More...
        const  	CLIENT_SSL =  2048;

        /// Client only flag. More...
        const  	CLIENT_IGNORE_SIGPIPE =  4096;

        /// Client knows about transactions. More...
        const  	CLIENT_TRANSACTIONS =  8192;

        /// DEPRECATED: Old flag for 4.1 protocol
        const  	CLIENT_RESERVED =  16384;

        /// DEPRECATED: Old flag for 4.1 authentication \ CLIENT_SECURE_CONNECTION. More...
        const  	CLIENT_RESERVED2 =   32768;

        /// Enable/disable multi-stmt support. More...
        const  	CLIENT_MULTI_STATEMENTS =  (1u32 << 16);

        /// Enable/disable multi-results. More...
        const  	CLIENT_MULTI_RESULTS =  (1u32 << 17);

        /// Multi-results and OUT parameters in PS-protocol. More...
        const  	CLIENT_PS_MULTI_RESULTS =  (1u32 << 18);

        /// Client supports plugin authentication. More...
        const  	CLIENT_PLUGIN_AUTH =  (1u32 << 19);

        /// Client supports connection attributes. More...
        const  	CLIENT_CONNECT_ATTRS =  (1u32 << 20);

        /// Enable authentication response packet to be larger than 255 bytes. More...
        const  	CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA =  (1u32 << 21);

        /// Don't close the connection for a user account with expired password. More...
        const  	CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS =  (1u32 << 22);

        /// Capable of handling server state change information. More...
        const  	CLIENT_SESSION_TRACK =  (1u32 << 23);

        /// Client no longer needs EOF_Packet and will use OK_Packet instead. More...
        const  	CLIENT_DEPRECATE_EOF =  (1u32 << 24);

        /// The client can handle optional metadata information in the resultset. More...
        const  	CLIENT_OPTIONAL_RESULTSET_METADATA =  (1u32 << 25);

        /// Compression protocol extended to support zstd compression method. More...
        const  	CLIENT_ZSTD_COMPRESSION_ALGORITHM =  (1u32 << 26);

        /// Support optional extension for query parameters into the COM_QUERY and COM_STMT_EXECUTE packets. More...
        const  	CLIENT_QUERY_ATTRIBUTES =  (1u32 << 27);

        /// Support Multi factor authentication. More...
        const  	MULTI_FACTOR_AUTHENTICATION =  (1u32 << 28);

        /// This flag will be reserved to extend the 32bit capabilities structure to 64bits. More...
        const  	CLIENT_CAPABILITY_EXTENSION =  (1u32 << 29);

        /// Verify server certificate. More...
        const  	CLIENT_SSL_VERIFY_SERVER_CERT =  (1u32 << 30);

        /// Don't reset the options after an unsuccessful connect. More...
        const  	CLIENT_REMEMBER_OPTIONS =  (1u32 << 31);
    }
}
