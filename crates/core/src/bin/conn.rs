use std::{
    io::{Read, Write},
    net::TcpStream,
};

use boxercrab::{
    codec::{CheckedBuf, Decode, DecodeError, DecodeResult, Encode, Int1, Int3, Int4},
    connector::{
        decode_packet, encode_packet, AuthSwitchReq, Capabilities, HandshakeResponse41,
        HandshakeV10, Packet,
    },
};
use bytes::{Buf, Bytes, BytesMut};

fn main() {
    let stream = TcpStream::connect("127.0.0.1:3306").unwrap();
    let mut socket = MysqlSocket::new(stream);
    let handshake: Packet<HandshakeV10> = socket.read_packet().unwrap();
    println!("{handshake:?}");
    let resp = HandshakeResponse41 {
        caps: Capabilities::CLIENT_LONG_PASSWORD
            | Capabilities::CLIENT_PROTOCOL_41
            | Capabilities::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA
            | Capabilities::CLIENT_PLUGIN_AUTH,
        max_packet_size: Int4::from(1 << 16),
        charset: Int1::from(255),
        user_name: "test".into(),
        auth_resp: BytesMut::new(),
        database: None,
        plugin_name: Some("mysql_native_password".into()),
        connect_attrs: Default::default(),
        zstd_level: Int1::from(0),
    };
    socket.write_packet(1, &resp).unwrap();
    let switch_req: Packet<AuthSwitchReq> = socket.read_packet().unwrap();
    println!("{switch_req:?}");
    std::thread::sleep(std::time::Duration::from_secs(10));
}

pub struct MysqlSocket {
    stream: TcpStream,
}

#[derive(Debug)]
pub enum PacketError {
    IOError(std::io::Error),
    Decode(DecodeError),
}

impl From<std::io::Error> for PacketError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<DecodeError> for PacketError {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
    }
}

impl MysqlSocket {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn read_packet<P: Decode<BytesMut>>(&mut self) -> Result<Packet<P>, PacketError> {
        let mut buf = BytesMut::new();
        buf.resize(4, 0);
        self.stream.read_exact(&mut buf)?;
        let len = Int3::decode(&mut buf)?;
        let seq_id = Int1::decode(&mut buf)?;
        let mut buf = BytesMut::with_capacity(len.int() as usize);
        buf.resize(len.int() as usize, 0);
        self.stream.read_exact(&mut buf)?;
        let payload = P::decode(&mut buf)?;
        Ok(Packet {
            len,
            seq_id,
            payload,
        })
    }

    pub fn write_packet<P: Encode>(
        &mut self,
        seq_id: u8,
        payload: &P,
    ) -> Result<(), std::io::Error> {
        let mut buf = BytesMut::new();
        encode_packet(seq_id, payload, &mut buf);
        self.stream.write_all(&buf)
    }
}
