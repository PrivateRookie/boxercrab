use super::{Event, Header, extract_string};
use nom::{number::complete::{le_u32, le_u16, le_u8}, bytes::complete::take, IResult, combinator::map};

// source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L295-L344
// event_data layout: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L387-L416
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FormatDesc {
    header: Header,
    binlog_version: u16,
    mysql_server_version: String,
    create_timestamp: u32,
    event_header_length: u8,
    supported_types: Vec<u8>,
    checksum_alg: u8,
    checksum: u32,
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, binlog_version) = le_u16(input)?;
    let (i, mysql_server_version) = map(take(50usize), |s: &[u8]| extract_string(s))(i)?;
    let (i, create_timestamp) = le_u32(i)?;
    let (i, event_header_length) = le_u8(i)?;
    let num = header.event_size - 19 - (2 + 50 + 4 + 1) - 1 - 4;
    let (i, supported_types) = map(take(num), |s: &[u8]| s.to_vec())(i)?;
    let (i, checksum_alg) = le_u8(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::FormatDesc(FormatDesc {
            header,
            binlog_version,
            mysql_server_version,
            create_timestamp,
            event_header_length,
            supported_types,
            checksum_alg,
            checksum,
        }),
    ))
}
