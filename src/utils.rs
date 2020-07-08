use nom::{
    bytes::complete::{take, take_till},
    combinator::map,
    number::complete::{le_u16, le_u32, le_u64, le_u8},
    IResult,
};

/// parse fixed len int
///
/// ref: https://dev.mysql.com/doc/internals/en/integer.html#fixed-length-integer
pub fn int_fixed<'a>(input: &'a [u8], len: u8) -> IResult<&'a [u8], u64> {
    match len {
        1 => map(le_u8, |v| v as u64)(input),
        2 => map(le_u16, |v| v as u64)(input),
        3 | 6 => map(take(3usize), |s: &[u8]| {
            let mut filled = s.to_vec();
            if len == 3 {
                filled.extend(vec![0, 0, 0, 0, 0]);
            } else {
                filled.extend(vec![0, 0]);
            }
            pu64(&filled).unwrap().1
        })(input),
        4 => map(le_u32, |v| v as u64)(input),
        8 => le_u64(input),
        _ => unreachable!(),
    }
}

/// parse len encoded int, return (used_bytes, value).
///
/// ref: https://dev.mysql.com/doc/internals/en/integer.html#packet-Protocol::LengthEncodedInteger
pub fn int_lenenc<'a>(input: &'a [u8]) -> IResult<&'a [u8], (usize, u64)> {
    match input[0] {
        0..=0xfa => map(le_u8, |num: u8| (1, num as u64))(input),
        0xfb | 0xfc => {
            let (i, _) = take(1usize)(input)?;
            map(le_u16, |num: u16| (3, num as u64))(i)
        }
        0xfd => {
            let (i, _) = take(1usize)(input)?;
            let (i, v) = map(take(3usize), |s: &[u8]| {
                let mut raw = s.to_vec();
                raw.push(0);
                raw
            })(i)?;
            let (_, num) = pu32(&v).unwrap();
            Ok((i, (4, num as u64)))
        }
        0xfe => {
            let (i, _) = take(1usize)(input)?;
            map(le_u64, |v: u64| (9, v))(i)
        }
        0xff => unreachable!(),
    }
}

/// parse length encoded string
///
/// ref: https://dev.mysql.com/doc/internals/en/string.html#packet-Protocol::LengthEncodedString
pub fn string_lenenc<'a>(input: &'a [u8]) -> IResult<&'a [u8], String> {
    let (i, (_, str_len)) = int_lenenc(input)?;
    map(take(str_len), |s: &[u8]| {
        String::from_utf8_lossy(s).to_string()
    })(i)
}

/// parse null terminated string, consume null byte
///
/// ref: https://dev.mysql.com/doc/internals/en/string.html#packet-Protocol::NulTerminatedString
pub fn string_nul(input: &[u8]) -> IResult<&[u8], String> {
    let (i, ret) = map(take_till(|c: u8| c == 0x00), |s| {
        String::from_utf8_lossy(s).to_string()
    })(input)?;
    let (i, _) = take(1usize)(i)?;
    Ok((i, ret))
}

/// extract n(n <= len(input)) bytes string
pub fn extract_string(input: &[u8]) -> String {
    let null_end = input
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(input.len());
    String::from_utf8_lossy(&input[0..null_end]).to_string()
}

/// extract len bytes string
///
/// ref: https://dev.mysql.com/doc/internals/en/string.html#packet-Protocol::VariableLengthString
pub fn string_var(input: &[u8], len: usize) -> String {
    if input.len() <= len {
        String::from_utf8_lossy(&input).to_string()
    } else {
        String::from_utf8_lossy(&input[0..len]).to_string()
    }
}

/// parse fixed len string.
///
/// ref: https://dev.mysql.com/doc/internals/en/string.html#packet-Protocol::FixedLengthString
pub fn string_fixed(input: &[u8]) -> IResult<&[u8], (u8, String)> {
    let (i, len) = le_u8(input)?;
    map(take(len), move |s: &[u8]| {
        (len, String::from_utf8_lossy(s).to_string())
    })(i)
}

pub fn pu32(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

pub fn pu64(input: &[u8]) -> IResult<&[u8], u64> {
    le_u64(input)
}
