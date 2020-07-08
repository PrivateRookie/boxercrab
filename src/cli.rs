use boxercrab::events::{check_start, Event};
use log4rs;

fn main() {
    log4rs::init_file("config/log.yaml", Default::default()).unwrap();
    let data = include_bytes!("../tests/bin_files/binlog.bin").clone();
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
