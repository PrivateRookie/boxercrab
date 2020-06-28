use super::{Event, Header};
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

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, gtid_sets) = map(take(header.event_size - 19 - 4 - 4), |s: &[u8]| s.to_vec())(input)?;
    let (i, buf_size) = le_u32(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::PreviousGtids(PreviousGtids {
            header,
            gtid_sets,
            buf_size,
            checksum,
        }),
    ))
}
