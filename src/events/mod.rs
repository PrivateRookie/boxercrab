use nom::{
    number::complete::{le_u16, le_u32, le_u8},
    IResult,
};

mod anonymous_gtid;
mod format_desc;
mod previous_gtids;
mod query;
mod table_map;

fn extract_string(input: &[u8]) -> String {
    let null_end = input.iter().position(|&c| c == b'\0').unwrap_or(input.len());
    String::from_utf8(input[0..null_end].to_vec()).unwrap()
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    pub timestamp: u32,
    pub event_type: u8,
    pub server_id: u32,
    pub event_size: u32,
    pub log_pos: u32,
    pub flags: u16,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (i, timestamp) = le_u32(input)?;
    let (i, event_type) = le_u8(i)?;
    let (i, server_id) = le_u32(i)?;
    let (i, event_size) = le_u32(i)?;
    let (i, log_pos) = le_u32(i)?;
    let (i, flags) = le_u16(i)?;
    Ok((
        i,
        Header {
            timestamp,
            event_type,
            server_id,
            event_size,
            log_pos,
            flags,
        },
    ))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    Query(query::Query),
    FormatDesc(format_desc::FormatDesc),
    AnonymousGtid(anonymous_gtid::AnonymousGtid),
    PreviousGtids(previous_gtids::PreviousGtids),
    TableMap(table_map::TableMap)
}

impl Event {
    pub fn parse<'a>(input: &'a [u8]) -> IResult<&'a [u8], Event> {
        let (input, header) = parse_header(input)?;
        match header.event_type {
            0x02 => query::parse(input, header),
            0x0f => format_desc::parse(input, header),
            0x13 => table_map::parse(input, header),
            0x22 => anonymous_gtid::parse(input, header),
            0x23 => previous_gtids::parse(input, header),
            _ => unreachable!(),
        }
    }
}
