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
            7 => ColTypes::Timestamp,
            8 => ColTypes::LongLong,
            9 => ColTypes::Int24,
            10 => ColTypes::Date,
            11 => ColTypes::Time,
            12 => ColTypes::DateTime,
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
            ColTypes::VarChar(_) => map(le_u16, |v| (1, ColTypes::VarChar(v)))(input),
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

    pub fn parse<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
        let len: u16 = match *self {
            ColTypes::Decimal => 4,
            ColTypes::Tiny => 1,
            ColTypes::Short => 2,
            ColTypes::Long => 4,
            ColTypes::Float(_) => 4,
            ColTypes::Double(_) => 8,
            ColTypes::Null => 0,
            ColTypes::Timestamp => 1,
            ColTypes::LongLong => 8,
            ColTypes::Int24 => 4,
            ColTypes::Date => 1,
            ColTypes::Time => 1,
            ColTypes::DateTime => 1,
            ColTypes::Year => 2,
            ColTypes::NewDate => 0,
            ColTypes::VarChar(len) => len,
            ColTypes::Bit(b1, b2) => ((b1 + 7) / 8 + (b2 + 7) / 8) as u16,
            ColTypes::NewDecimal(_, _) => 8,
            ColTypes::Enum => 0,
            ColTypes::Set => 0,
            ColTypes::TinyBlob => 0,
            ColTypes::MediumBlob => 0,
            ColTypes::LongBlob => 0,
            ColTypes::Blob(len) => len as u16,
            ColTypes::VarString(_, len) => len as u16,
            ColTypes::String(_, len) => len as u16,
            ColTypes::Geometry(len) => len as u16,
        };
        let (i, data) = take(len)(input)?;
        match *self {
            ColTypes::Time | ColTypes::Date | ColTypes::DateTime | ColTypes::Timestamp => {
                let mut ret = data.to_vec();
                let (_, len) = le_u8(data)?;
                let (i, data) = take(len)(i)?;
                ret.extend(data);
                Ok((i, ret))
            }
            _ => Ok((i, data.to_vec())),
        }
    }
}
