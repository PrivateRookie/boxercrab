use super::{Event, Header};
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_i32, le_u32, le_u8},
    IResult,
};

// source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L932-L991
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnonymousGtid {
    header: Header,
    last_committed: i32,
    sequence_number: i32,
    rbr_only: bool,
    // FIXME unknown field ???
    buf: Vec<u8>,
    checksum: u32,
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, last_committed) = le_i32(input)?;
    let (i, sequence_number) = le_i32(i)?;
    let (i, rbr_only) = map(le_u8, |t: u8| t == 1)(i)?;
    let (i, buf) = map(
        take(header.event_size - 19 - (4 + 4 + 1 + 4)),
        |s: &[u8]| s.to_vec(),
    )(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::AnonymousGtid(AnonymousGtid {
            header,
            last_committed,
            sequence_number,
            rbr_only,
            buf,
            checksum,
        }),
    ))
}
