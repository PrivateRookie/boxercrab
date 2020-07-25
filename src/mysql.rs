use crate::utils::pu32;
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_u16, le_u8},
    sequence::tuple,
    IResult,
};
use serde::Serialize;

/// type def ref: https://dev.mysql.com/doc/internals/en/table-map-event.html
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum ColTypes {
    Decimal,
    Tiny,
    Short,
    Long,
    Float(u8),
    Double(u8),
    Null,
    Timestamp,
    LongLong,
    Int24,
    Date,
    Time,
    DateTime,
    Year,
    NewDate, // internal used
    VarChar(u16),
    Bit(u8, u8),
    NewDecimal(u8, u8),
    Enum,       // internal used
    Set,        // internal used
    TinyBlob,   // internal used
    MediumBlob, // internal used
    LongBlob,   // internal used
    Blob(u8),
    VarString(u8, u8),
    String(u8, u8),
    Geometry(u8),
}

impl ColTypes {
    /// return (identifer, bytes used) of column type
    pub fn meta(&self) -> (u8, u8) {
        match *self {
            ColTypes::Decimal => (0, 0),
            ColTypes::Tiny => (1, 0),
            ColTypes::Short => (2, 0),
            ColTypes::Long => (3, 0),
            ColTypes::Float(_) => (4, 1),
            ColTypes::Double(_) => (5, 1),
            ColTypes::Null => (6, 0),
            ColTypes::Timestamp => (7, 0),
            ColTypes::LongLong => (8, 0),
            ColTypes::Int24 => (9, 0),
            ColTypes::Date => (10, 0),
            ColTypes::Time => (11, 0),
            ColTypes::DateTime => (12, 0),
            ColTypes::Year => (13, 0),
            ColTypes::NewDate => (14, 0),
            ColTypes::VarChar(_) => (15, 2),
            ColTypes::Bit(_, _) => (16, 2),
            ColTypes::NewDecimal(_, _) => (246, 2),
            ColTypes::Enum => (247, 0),
            ColTypes::Set => (248, 0),
            ColTypes::TinyBlob => (249, 0),
            ColTypes::MediumBlob => (250, 0),
            ColTypes::LongBlob => (251, 0),
            ColTypes::Blob(_) => (252, 1),
            ColTypes::VarString(_, _) => (253, 2),
            ColTypes::String(_, _) => (254, 2),
            ColTypes::Geometry(_) => (255, 1),
        }
    }

    pub fn from_u8(t: u8) -> Self {
        match t {
            0 => ColTypes::Decimal,
            1 => ColTypes::Tiny,
            2 => ColTypes::Short,
            3 => ColTypes::Long,
            4 => ColTypes::Float(4),
            5 => ColTypes::Double(8),
            6 => ColTypes::Null,
            7 | 17 => ColTypes::Timestamp,
            8 => ColTypes::LongLong,
            9 => ColTypes::Int24,
            10 => ColTypes::Date,
            11 | 19 => ColTypes::Time,
            12 | 18 => ColTypes::DateTime,
            13 => ColTypes::Year,
            14 => ColTypes::NewDate,
            15 => ColTypes::VarChar(0),
            16 => ColTypes::Bit(0, 0),
            246 => ColTypes::NewDecimal(10, 0),
            247 => ColTypes::Enum,
            248 => ColTypes::Set,
            249 => ColTypes::TinyBlob,
            250 => ColTypes::MediumBlob,
            251 => ColTypes::LongBlob,
            252 => ColTypes::Blob(1),
            253 => ColTypes::VarString(1, 0),
            254 => ColTypes::String(253, 0),
            255 => ColTypes::Geometry(1),
            _ => {
                log::error!("unknown column type: {}", t);
                unreachable!()
            }
        }
    }

    pub fn parse_def<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (usize, Self)> {
        match *self {
            ColTypes::Float(_) => map(le_u8, |v| (1, ColTypes::Float(v)))(input),
            ColTypes::Double(_) => map(le_u8, |v| (1, ColTypes::Double(v)))(input),
            ColTypes::VarChar(_) => map(le_u16, |v| (2, ColTypes::VarChar(v)))(input),
            ColTypes::NewDecimal(_, _) => map(tuple((le_u8, le_u8)), |(m, d)| {
                (2, ColTypes::NewDecimal(m, d))
            })(input),
            ColTypes::Blob(_) => map(le_u8, |v| (1, ColTypes::Blob(v)))(input),
            ColTypes::VarString(_, _) => map(tuple((le_u8, le_u8)), |(t, len)| {
                (2, ColTypes::VarString(t, len))
            })(input),
            ColTypes::String(_, _) => map(tuple((le_u8, le_u8)), |(t, len)| {
                (2, ColTypes::String(t, len))
            })(input),
            ColTypes::Bit(_, _) => {
                map(tuple((le_u8, le_u8)), |(b1, b2)| (2, ColTypes::Bit(b1, b2)))(input)
            }
            ColTypes::Geometry(_) => map(le_u8, |v| (1, ColTypes::Geometry(v)))(input),
            _ => Ok((input, (0, self.clone()))),
        }
    }

    pub fn parse<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (usize, ColValues)> {
        match *self {
            ColTypes::Decimal => {
                map(take(4usize), |s: &[u8]| (4, ColValues::Decimal(s.to_vec())))(input)
            }
            ColTypes::Tiny => map(take(1usize), |s: &[u8]| (1, ColValues::Tiny(s.to_vec())))(input),
            ColTypes::Short => {
                map(take(2usize), |s: &[u8]| (2, ColValues::Short(s.to_vec())))(input)
            }
            ColTypes::Long => map(take(4usize), |s: &[u8]| (4, ColValues::Long(s.to_vec())))(input),
            ColTypes::Float(_) => {
                map(take(4usize), |s: &[u8]| (4, ColValues::Float(s.to_vec())))(input)
            }
            ColTypes::Double(_) => {
                map(take(8usize), |s: &[u8]| (8, ColValues::Double(s.to_vec())))(input)
            }
            ColTypes::Null => map(take(0usize), |_| (0, ColValues::Null))(input),
            ColTypes::LongLong => map(take(8usize), |s: &[u8]| {
                (8, ColValues::LongLong(s.to_vec()))
            })(input),
            ColTypes::Int24 => {
                map(take(4usize), |s: &[u8]| (4, ColValues::Int24(s.to_vec())))(input)
            }
            ColTypes::Timestamp => map(parse_packed, |(len, v): (usize, Vec<u8>)| {
                (len, ColValues::Timestamp(v))
            })(input),
            ColTypes::Date => map(parse_packed, |(len, v): (usize, Vec<u8>)| {
                (len, ColValues::Date(v))
            })(input),
            ColTypes::Time => map(parse_packed, |(len, v): (usize, Vec<u8>)| {
                (len, ColValues::Time(v))
            })(input),
            ColTypes::DateTime => map(parse_packed, |(len, v): (usize, Vec<u8>)| {
                (len, ColValues::DateTime(v))
            })(input),
            ColTypes::Year => map(take(2usize), |s: &[u8]| (2, ColValues::Year(s.to_vec())))(input),
            ColTypes::NewDate => map(take(0usize), |_| (0, ColValues::NewDate))(input),
            // ref: https://dev.mysql.com/doc/refman/5.7/en/char.html
            ColTypes::VarChar(max_len) => {
                if max_len > 255 {
                    let (i, len) = le_u16(input)?;
                    map(take(len), move |s: &[u8]| {
                        (len as usize + 2, ColValues::VarChar(s.to_vec()))
                    })(i)
                } else {
                    let (i, len) = le_u8(input)?;
                    map(take(len), move |s: &[u8]| {
                        (len as usize + 1, ColValues::VarChar(s.to_vec()))
                    })(i)
                }
            }
            ColTypes::Bit(b1, b2) => {
                let len = ((b1 + 7) / 8 + (b2 + 7) / 8) as usize;
                map(take(len), move |s: &[u8]| (len, ColValues::Bit(s.to_vec())))(input)
            }
            ColTypes::NewDecimal(_, _) => map(take(8usize), |s: &[u8]| {
                (8, ColValues::NewDecimal(s.to_vec()))
            })(input),
            ColTypes::Enum => map(take(0usize), |_| (0, ColValues::Enum))(input),
            ColTypes::Set => map(take(0usize), |_| (0, ColValues::Set))(input),
            ColTypes::TinyBlob => map(take(0usize), |_| (0, ColValues::TinyBlob))(input),
            ColTypes::MediumBlob => map(take(0usize), |_| (0, ColValues::MediumBlob))(input),
            ColTypes::LongBlob => map(take(0usize), |_| (0, ColValues::LongBlob))(input),
            ColTypes::Blob(len_bytes) => {
                let mut raw_len = input[..len_bytes as usize].to_vec();
                for _ in 0..(4 - len_bytes) {
                    raw_len.push(0);
                }
                let (_, len) = pu32(&raw_len).unwrap();
                map(take(len), move |s: &[u8]| {
                    (
                        len_bytes as usize + len as usize,
                        ColValues::Blob(s.to_vec()),
                    )
                })(&input[len_bytes as usize..])
            }
            ColTypes::VarString(_, _) => {
                // TODO should check string max_len ?
                let (i, len) = le_u8(input)?;
                map(take(len), move |s: &[u8]| {
                    (len as usize, ColValues::VarString(s.to_vec()))
                })(i)
            }
            ColTypes::String(_, _) => {
                // TODO should check string max_len ?
                let (i, len) = le_u8(input)?;
                map(take(len), move |s: &[u8]| {
                    (len as usize, ColValues::VarChar(s.to_vec()))
                })(i)
            }
            // TODO fix do not use len in def ?
            ColTypes::Geometry(len) => map(take(len), |s: &[u8]| {
                (len as usize, ColValues::Geometry(s.to_vec()))
            })(input),
        }
    }
}

fn parse_packed(input: &[u8]) -> IResult<&[u8], (usize, Vec<u8>)> {
    let mut data = vec![input[0]];
    let (i, len) = le_u8(input)?;
    let (i, raw) = take(len)(i)?;
    data.extend(raw);
    Ok((i, (len as usize + 1, data)))
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub enum ColValues {
    Decimal(Vec<u8>),
    Tiny(Vec<u8>),
    Short(Vec<u8>),
    Long(Vec<u8>),
    Float(Vec<u8>),
    Double(Vec<u8>),
    Null,
    Timestamp(Vec<u8>),
    LongLong(Vec<u8>),
    Int24(Vec<u8>),
    Date(Vec<u8>),
    Time(Vec<u8>),
    DateTime(Vec<u8>),
    Year(Vec<u8>),
    NewDate, // internal used
    VarChar(Vec<u8>),
    Bit(Vec<u8>),
    NewDecimal(Vec<u8>),
    Enum,       // internal used
    Set,        // internal used
    TinyBlob,   // internal used
    MediumBlob, // internal used
    LongBlob,   // internal used
    Blob(Vec<u8>),
    VarString(Vec<u8>),
    String(Vec<u8>),
    Geometry(Vec<u8>),
}
