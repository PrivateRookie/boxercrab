#![allow(non_camel_case_types)]

use events::{check_start, Event};
use log4rs;

mod events;
mod utils;

fn main() {
    log4rs::init_file("config/log.yaml", Default::default()).unwrap();
    let data = include_bytes!("../tests/binlog.bin").clone();
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
    println!("{:x?}", input);
}
