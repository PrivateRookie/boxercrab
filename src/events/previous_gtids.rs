use super::{Event, Header, Parse};
use nom::{bytes::complete::take, combinator::map, number::complete::le_u32, IResult};

// source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L1073-L1103
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PreviousGtids {
    header: Header,
    // FIXME this field may be wrong
    gtid_sets: Vec<u8>,
    buf_size: u32,
    checksum: u32,
}

impl Parse<PreviousGtids> for PreviousGtids {
    fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], PreviousGtids> {
        let (i, gtid_sets) =
            map(take(header.event_size - 19 - 4 - 4), |s: &[u8]| s.to_vec())(input)?;
        let (i, buf_size) = le_u32(i)?;
        let (i, checksum) = le_u32(i)?;
        Ok((
            i,
            PreviousGtids {
                header,
                gtid_sets,
                buf_size,
                checksum,
            },
        ))
    }
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let f = move |i| PreviousGtids::parse(i, header.clone());
    map(f, |e| Event::PreviousGtids(e))(input)
}

#[test]
fn test_previous_gtids() {
    use super::parse_header;

    let input: Vec<u8> = vec![
        220, 156, 253, 94, 35, 123, 0, 0, 0, 31, 0, 0, 0, 154, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 82, 75, 196, 253,
    ];
    let (i, header) = parse_header(&input).unwrap();
    let (i, _) = PreviousGtids::parse(i, header).unwrap();
    assert_eq!(i.len(), 0);
    // TODO do more parse
}
