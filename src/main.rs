#![allow(non_camel_case_types)]

use events::*;
use nom::{bytes::complete::tag, IResult};

pub mod events;

fn check_start(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag([254, 98, 105, 110])(i)
}

fn main() {
    // let data = include_bytes!("binlog.bin").clone();
    // let (input, _) = check_start(&data).unwrap();
    // let (input, _) = Event::parse(input).unwrap();
    // let (input, _) = Event::parse(input).unwrap();
    // let (input, _) = Event::parse(input).unwrap();
    // let (input, e) = Event::parse(input).unwrap();
    // println!("{:x?}", e);
    // println!("{:x?}", input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_binlog() {
        assert_eq!(
            check_start(".bin".as_bytes()),
            Ok(("".as_bytes(), ".bin".as_bytes()))
        );
    }
}
