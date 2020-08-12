use bytes::{BufMut, Bytes, BytesMut};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::Connection as SQLConnection;
use sqlx::MySqlConnection;
use std::error::Error;

#[derive(Debug)]
pub struct Connection {
    conn: Option<MySqlConnection>,
    has_requested: bool,
    binlog_file: String,
    pub url: String,
    pub id: u32,
}

impl Connection {
    pub fn new(url: String, id: u32) -> Self {
        Connection {
            conn: None,
            has_requested: false,
            binlog_file: String::new(),
            url,
            id,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let mut conn = MySqlConnection::connect(&self.url).await?;
        conn.ping().await?;

        // send a query to tell master we can handle checksum
        let mut enable_checksum = BytesMut::with_capacity(100);
        enable_checksum.put_u8(0x03);
        enable_checksum.put(&b"set @master_binlog_checksum= @@global.binlog_checksum"[..]);
        conn.stream.send_packet(enable_checksum.as_ref()).await?;

        self.conn = Some(conn);
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Bytes, Box<dyn Error>> {
        if self.conn.is_none() {
            self.connect().await?;
        }

        if !self.has_requested {
            // query server current position
            let pool = MySqlPoolOptions::new().connect(&self.url).await?;
            let (binlog_file, position, ..): (String, u32, String, String, String) =
                sqlx::query_as("show master status")
                    .fetch_one(&pool)
                    .await?;

            // send COM_BINLOG_DUMP command
            let mut com_bindump = BytesMut::new();
            // COM identifier
            com_bindump.put_u8(0x12);
            // binlog position
            com_bindump.put_u32_le(position);
            // command flags, always be blocking here
            com_bindump.put_u16_le(0);
            // client id
            com_bindump.put_u32_le(self.id);
            // binlog file name
            com_bindump.put(binlog_file.as_bytes());
            println!("{:x?}", com_bindump.as_ref());

            self.conn
                .as_mut()
                .unwrap()
                .stream
                .send_packet(com_bindump.as_ref())
                .await?;

            self.has_requested = true;
        }

        let bytes = self.conn.as_mut().unwrap().stream.recv_packet().await?;
        Ok(bytes.0)
    }
}
