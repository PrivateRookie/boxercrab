use bytes::BytesMut;

pub trait Decode: Sized {
    fn parse(buf: &mut BytesMut) -> Result<Self, ParseError>;
}

use thiserror::Error;
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("no enough data")]
    NoEnoughData,
    #[error("invalid utf-8 string")]
    InvalidUtf8,
    #[error("missing terminal null bytes")]
    MissingNull,
    #[error("invalid data")]
    InvalidData,
}
