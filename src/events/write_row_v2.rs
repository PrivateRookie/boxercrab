use super::{pu64, Event, Header};
use crate::utils::{extract_string, lenenc_int};
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_u16, le_u8},
    IResult,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Flags {
    pub end_of_stmt: bool,
    pub foreign_key_checks: bool,
    pub unique_key_checks: bool,
    pub has_columns: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtraData {
    pub d_type: ExtraDataType,
    pub data: Payload,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExtraDataType {
    RW_V_EXTRAINFO_TAG = 0x00,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Payload {
    ExtraDataInfo {
        length: u8,
        format: ExtraDataFormat,
        payload: String,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum ExtraDataFormat {
    NDB = 0x00,
    OPEN1 = 0x40,
    OPEN2 = 0x41,
    MULTI = 0xff,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Row {
    pub null_bit_mask: Vec<u8>,
    pub values: Vec<u8>,
}

pub fn parse_extra_data<'a>(input: &'a [u8]) -> IResult<&'a [u8], ExtraData> {
    let (i, d_type) = map(le_u8, |t: u8| match t {
        0x00 => ExtraDataType::RW_V_EXTRAINFO_TAG,
        _ => {
            log::error!("unknown extra data type {}", t);
            unreachable!()
        }
    })(input)?;
    let (i, length) = le_u8(i)?;
    let (i, extra_data_format) = map(le_u8, |fmt: u8| match fmt {
        0x00 => ExtraDataFormat::NDB,
        0x40 => ExtraDataFormat::OPEN1,
        0x41 => ExtraDataFormat::OPEN2,
        0xff => ExtraDataFormat::MULTI,
        _ => {
            dbg!(&fmt);
            log::error!("unknown extract data format {}", fmt);
            unreachable!()
        }
    })(i)?;
    let (i, payload) = map(take(length), |s: &[u8]| extract_string(s))(i)?;
    Ok((
        i,
        ExtraData {
            d_type,
            data: Payload::ExtraDataInfo {
                length,
                format: extra_data_format,
                payload,
            },
        },
    ))
}

