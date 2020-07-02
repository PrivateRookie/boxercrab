use super::{Event, Header};
use crate::utils::extract_string;
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_u16, le_u32, le_u8},
    IResult,
};

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

// #[test]
// fn test_format_desc() {
//     use super::parse_header;
//     let input: Vec<u8> = vec![
//         0x4, 0xdc, 0x9c, 0xfd, 0x5e, 0x0f, 0x7b, 0x00, 0x00, 0x00, 0x77, 0x00, 0x00, 0x00, 0x7b,
//         0x00, 0x00, 0x00, 0x01, 0x00, 0x17, 0x04, 0x00, 0x35, 0x2e, 0x37, 0x2e, 0x32, 0x39, 0x2d,
//         0x6c, 0x6f, 0x67, 0x00, 0x00, 0x00, 0x00, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x37, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x47, 0x00, 0x00, 0x00,
//         0x00, 0xdc, 0x9c, 0xfd, 0x5e, 0x13, 0x38, 0x0d, 0x00, 0x08, 0x00, 0x12, 0x00, 0x57, 0x04,
//         0x04, 0x04, 0x04, 0x12, 0x00, 0x00, 0x5f, 0x00, 0x04, 0x1a, 0x08, 0x00, 0x00, 0x00, 0x08,
//         0x67, 0x08, 0x08, 0x02, 0x00, 0x00, 0x00, 0x0a, 0x0a, 0x0a, 0x2a, 0x2a, 0x00, 0x12, 0x34,
//         0x00, 0x01, 0x77, 0xcf, 0x58, 0x7e, 0xee,
//     ];
//     let (i, header) = parse_header(&input).unwrap();
//     match parse(i, header) {
//         Ok((i, Event::FormatDesc(event))) => {}
//         Err(e) => println!("{:?}", e),
//     }
// }
