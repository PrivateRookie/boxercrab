#![allow(non_camel_case_types)]

use events::*;
use log4rs;
use nom::{bytes::complete::tag, IResult};

pub mod events;
pub mod utils;

fn check_start(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag([254, 98, 105, 110])(i)
}

fn main() {
    log4rs::init_file("config/log.yaml", Default::default()).unwrap();
    let data = include_bytes!("binlog.bin").clone();
    let (input, e) = check_start(&data).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:?}\n", e);
    let (input, e) = Event::parse(input).unwrap();
    println!("\n{:#x?}\n", e);
    // println!("{:x?}", input);
}
