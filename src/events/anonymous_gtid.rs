use super::{Event, Header, Parse};
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_i32, le_i64, le_u32, le_u64, le_u8},
    IResult,
};

// source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L932-L991
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnonymousGtid {
    header: Header,
    rbr_only: bool,
    encoded_sig_length: u32,
    encoded_gno_length: u32,
    // FIXME unknown fields
    unknown: Vec<u8>,
    last_committed: i64,
    sequence_number: i64,
    checksum: u32,
}

impl Parse<AnonymousGtid> for AnonymousGtid {
    fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], AnonymousGtid> {
        let (i, rbr_only) = map(le_u8, |t: u8| t == 0)(input)?;
        let (i, encoded_sig_length) = le_u32(i)?;
        let (i, encoded_gno_length) = le_u32(i)?;
        let (i, unknown) = map(
            take(header.event_size - 19 - (1 + 4 * 2 + 8 * 2 + 4)),
            |s: &[u8]| s.to_vec(),
        )(i)?;
        let (i, last_committed) = le_i64(i)?;
        let (i, sequence_number) = le_i64(i)?;
        let (i, checksum) = le_u32(i)?;
        Ok((
            i,
            AnonymousGtid {
                header,
                rbr_only,
                encoded_sig_length,
                encoded_gno_length,
                last_committed,
                sequence_number,
                unknown,
                checksum,
            },
        ))
    }
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let f = move |i| AnonymousGtid::parse(i, header.clone());
    map(f, |e| Event::AnonymousGtid(e))(input)
}

#[test]
fn test_anonymous_gtids() {
    use super::parse_header;
    let input: Vec<u8> = vec![
        54, 157, 253, 94, 34, 123, 0, 0, 0, 65, 0, 0, 0, 219, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 10, 21, 198, 18,
    ];
    let (i, header) = parse_header(&input).unwrap();
    let (i, event) = AnonymousGtid::parse(i, header).unwrap();
    assert_eq!(event.last_committed, 0);
    assert_eq!(event.sequence_number, 1);
    assert_eq!(event.rbr_only, false);
    assert_eq!(i.len(), 0);
}
