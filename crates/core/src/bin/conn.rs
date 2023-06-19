use std::{
    io::{Read, Write},
    net::TcpStream,
};

use boxercrab::{
    codec::{Decode, DecodeError, DecodeResult, Int1, Int3},
    connector::{
        decode_packet, encode_packet, Capabilities, HandshakeResponse41, HandshakeV10, Packet,
    },
};
use bytes::{Buf, BytesMut};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:3306").unwrap();
    let mut conn = Conn::new();
    let handshake: Packet<HandshakeV10> = conn.read(&mut stream);
}

pub struct Conn {
    data: BytesMut,
    buf: [u8; 4096],
}

impl Conn {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            buf: [0; 4096],
        }
    }

    pub fn read<R: Read, P: Decode<BytesMut>>(&mut self, stream: &mut R) -> Packet<P> {
        let packet = loop {
            let num = stream.read(&mut self.buf).unwrap();
            self.data.copy_from_slice(&self.buf[..num]);
            let maybe: DecodeResult<Packet<P>> = decode_packet(&mut self.data);
            if let Err(DecodeError::NoEnoughData) = maybe {
                continue;
            }
            break maybe.unwrap();
        };
        packet
    }
}

// let resp = HandshakeResponse41 {
//     caps: Capabilities::CLIENT_LONG_PASSWORD
//         | Capabilities::CLIENT_PROTOCOL_41
//         | Capabilities::CLIENT_PLUGIN_AUTH
//         | Capabilities::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA,
//     max_packet_size: Int3::from(1 << 16),
//     charset: Int1::from(255),
//     user_name: "test".into(),
//     auth_resp: BytesMut::new(),
//     database: None,
//     plugin_name: Some("mysql_native_password".into()),
//     connect_attrs: Default::default(),
//     zstd_level: Int1::from(0),
// };
// let mut r = BytesMut::new();
// encode_packet(1, &resp, &mut r);
// stream.write_all(&r).unwrap();
