use std::{
    io::{Read, Write},
    net::TcpStream,
};

use boxercrab::{
    codec::{Decode, DecodeError, Encode, Int1, Int2, Int3, Int4, VLenInt},
    connector::{
        encode_packet, native_password_auth, AuthSwitchReq, AuthSwitchResp, Capabilities, ColDef,
        ComBinLogDump, ComQuery, ComQuit, HandshakeResponse41, HandshakeV10, OkOrErr, OkPacket,
        Packet, TextResult, TextResultSet,
    },
};
use bytes::BytesMut;

fn main() {
    let stream = TcpStream::connect("127.0.0.1:3306").unwrap();
    let mut socket = MysqlSocket::new(stream);
    let _handshake: Packet<HandshakeV10> = socket.read_packet().unwrap();
    let resp = HandshakeResponse41 {
        caps: Capabilities::CLIENT_LONG_PASSWORD
            | Capabilities::CLIENT_PROTOCOL_41
            | Capabilities::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA
            | Capabilities::CLIENT_RESERVED
            | Capabilities::CLIENT_RESERVED2
            | Capabilities::CLIENT_DEPRECATE_EOF
            | Capabilities::CLIENT_PLUGIN_AUTH,
        max_packet_size: Int4::from(1 << 24),
        charset: Int1::from(255),
        user_name: "auth".into(),
        auth_resp: BytesMut::new(),
        database: None,
        plugin_name: Some("mysql_native_password".into()),
        connect_attrs: Default::default(),
        zstd_level: Int1::from(0),
    };
    socket.write_packet(1, &resp).unwrap();
    let switch_req: AuthSwitchReq = socket.read_packet().unwrap().payload;
    if switch_req.plugin_name != "mysql_native_password" {
        panic!("")
    }
    let data = native_password_auth("1234".as_bytes(), &switch_req.plugin_data);
    let resp = AuthSwitchResp {
        data: BytesMut::from_iter(data),
    };
    socket.write_packet(3, &resp).unwrap();
    let _r: OkPacket = socket.read_packet().unwrap().payload;
    let query: ComQuery = "set @master_binlog_checksum= @@global.binlog_checksum".into();
    socket.write_packet(0, &query).unwrap();
    let _r: OkPacket = socket.read_packet().unwrap().payload;
    let query: ComQuery = "show master status".into();
    socket.write_packet(0, &query).unwrap();
    let result = socket.read_text_result_set().unwrap();
    let file = String::from_utf8(result.rows[0].columns[0].clone()).unwrap();
    let pos: u32 = String::from_utf8(result.rows[0].columns[1].clone())
        .unwrap()
        .parse()
        .unwrap();
    println!("{file} {pos}");
    // let _r: OkPacket = socket.read_packet().unwrap().payload;
    let dump = ComBinLogDump {
        pos: Int4::from(pos),
        flags: Int2::from(0),
        server_id: Int4::from(100),
        filename: file,
    };
    socket.write_packet(0, &dump).unwrap();
    loop {
        let buf: Vec<u8> = socket.read_packet().unwrap().payload;
        println!("{buf:x?}");
    }
    // println!("{oe:?}");
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

    pub fn read_text_result_set(&mut self) -> Result<TextResultSet, PacketError> {
        let column_count: VLenInt = self.read_packet()?.payload;
        let mut col_defs = vec![];
        for _ in 0..(column_count.int() as usize) {
            let col_def: ColDef = self.read_packet()?.payload;
            col_defs.push(col_def);
        }
        let mut rows = vec![];
        loop {
            let buf: Vec<u8> = self.read_packet()?.payload;
            let mut buf = BytesMut::from_iter(buf);
            if buf.first() == Some(&0xfe) && buf.len() < 9 {
                let _ = OkPacket::decode(&mut buf).unwrap();
                break;
            }
            let row = TextResult::decode(&mut buf).unwrap();
            rows.push(row);
        }
        Ok(TextResultSet {
            column_count,
            col_defs,
            rows,
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
