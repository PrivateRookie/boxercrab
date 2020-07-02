use super::{Event, Header, Parse};
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

impl Parse<FormatDesc> for FormatDesc {
    fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], FormatDesc> {
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
            FormatDesc {
                header,
                binlog_version,
                mysql_server_version,
                create_timestamp,
                event_header_length,
                supported_types,
                checksum_alg,
                checksum,
            },
        ))
    }
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let f = move |i| FormatDesc::parse(i, header.clone());
    map(f, |e| Event::FormatDesc(e))(input)
}

#[test]
fn test_format_desc() {
    use super::parse_header;
    let input: Vec<u8> = vec![
        220, 156, 253, 94, 15, 123, 0, 0, 0, 119, 0, 0, 0, 123, 0, 0, 0, 1, 0, 4, 0, 53, 46, 55,
        46, 50, 57, 45, 108, 111, 103, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 156, 253, 94, 19, 56, 13,
        0, 8, 0, 18, 0, 4, 4, 4, 4, 18, 0, 0, 95, 0, 4, 26, 8, 0, 0, 0, 8, 8, 8, 2, 0, 0, 0, 10,
        10, 10, 42, 42, 0, 18, 52, 0, 1, 207, 88, 126, 238,
    ];
    let (i, header) = parse_header(&input).unwrap();
    let (i, event) = FormatDesc::parse(i, header).unwrap();
    assert_eq!(event.binlog_version, 4);
    assert_eq!(event.mysql_server_version, "5.7.29-log");
    assert_eq!(event.create_timestamp, 1593679068);
    assert_eq!(i.len(), 0);
}
