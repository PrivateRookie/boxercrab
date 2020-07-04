use super::{pu64, Event, Header};
use crate::utils::{extract_string, parse_lenenc_int};
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

// pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
//     let (i, table_id): (&'a [u8], u64) = map(take(6usize), |id_raw: &[u8]| {
//         let mut filled = id_raw.to_vec();
//         filled.extend(vec![0, 0]);
//         pu64(&filled).unwrap().1
//     })(input)?;
//     let (i, flag) = map(le_u16, |flag: u16| {
//         Flag::try_from(flag).expect(&format!("unexpected flag: {}", flag))
//     })(i)?;
//     let (i, extra_data_len) = le_u16(i)?;
//     assert!(extra_data_len >= 2);
//     let (i, extra_data_type) = map(le_u8, |t: u8| {
//         ExtraDataType::try_from(t).expect(&format!("unexpected extra_data_type: {:x}", t))
//     })(i)?;
//     // only one type now
//     assert!(extra_data_type == ExtraDataType::RW_V_EXTRAINFO_TAG);
//     let (i, length) = le_u8(i)?;
//     let (i, extra_data_format) = map(le_u8, |f: u8| {
//         ExtraDataFormat::try_from(f).expect(&format!("unexpected format: {:x}", f))
//     })(i)?;
//     let (i, data_payload) = map(take(length), |s: &[u8]| extract_string(s))(i)?;
//     let extra_data = ExtraData {
//         _type: extra_data_type,
//         data: Payload::ExtraDataInfo {
//             length,
//             format: extra_data_format,
//             payload: data_payload
//         }
//     };
//     // parse body
//     let (i, column_count) = parse_lenenc_int(i)?;
//     let (i, column_present_bit_mask) = map(take((column_count + 7)/8), |s: &[u8]| s.to_vec())(i)?;

//     // parse row
//     let (i, null_bit_mask) = map(take(column_present_bit_mask.len()), |s: &[u8]| s.to_vec())(i)?;
//     let ()
// }
