pub mod codec;
pub mod connector;
pub mod binlog;

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
