use boxercrab::events::Event;
use log::LevelFilter;
use log4rs::{
    append::console::{ConsoleAppender, Target},
    config::{Appender, Config, Root},
    Handle,
};
use std::fs::File;
use std::io::prelude::*;
use structopt::{clap::arg_enum, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(name = "boxercrab-cli", about = "MySQL binlog tool impl with Rust")]
pub struct Args {
    /// enable debug info
    #[structopt(short, long)]
    debug: bool,

    #[structopt(subcommand)]
    sub: Cmd,
}

#[derive(Debug, StructOpt)]
enum Cmd {
    /// Transform a binlog file to specified format
    Serde {
        /// Binlog file path
        input: String,

        /// Output file path
        output: String,

        /// Output format
        #[structopt(short, long, possible_values = &Format::variants(), case_insensitive = true, default_value = "Json")]
        format: Format,
    },

    /// Show bin log desc msg
    Desc {
        /// Binlog file path
        input: String,
    },
}

arg_enum! {
    #[derive(Debug)]
    enum Format {
        Json,
        Yaml,
    }
}

fn init_log(debug: bool) -> Handle {
    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();
    log4rs::init_config(config).unwrap()
}

fn main() {
    let args = Args::from_args();
    let _handle = init_log(args.debug);
    match args.sub {
        Cmd::Serde {
            input,
            output,
            format,
        } => match File::open(&input) {
            Err(e) => println!("read {} error: {}", input, e),
            Ok(mut file) => {
                let mut buf = vec![];
                if let Ok(size) = file.read_to_end(&mut buf) {
                    log::debug!("read {} bytes", size);
                    println!("transform {} -> {} with {}", input, output, format);
                    match Event::from_bytes(&buf) {
                        Ok((_, data)) => {
                            if let Ok(mut output) = File::create(output) {
                                match format {
                                    Format::Json => {
                                        output
                                            .write_all(
                                                serde_json::to_string_pretty(&data)
                                                    .unwrap()
                                                    .as_bytes(),
                                            )
                                            .unwrap();
                                    }
                                    Format::Yaml => {
                                        output
                                            .write_all(
                                                serde_yaml::to_string(&data).unwrap().as_bytes(),
                                            )
                                            .unwrap();
                                    }
                                }
                            }
                        }
                        Err(e) => println!("invalid binlog file {:?}", e),
                    }
                }
            }
        },
        Cmd::Desc { input } => match File::open(&input) {
            Err(e) => println!("read {} error: {}", &input, e),
            Ok(mut file) => {
                let mut buf = vec![];
                if let Ok(size) = file.read_to_end(&mut buf) {
                    match Event::from_bytes(&buf) {
                        Ok((_, data)) => {
                            println!("File: {}, size: {}", input, size);
                            println!("Total: Events: {}", data.len());
                            match data.first().unwrap() {
                                Event::FormatDesc {
                                    binlog_version,
                                    mysql_server_version,
                                    create_timestamp,
                                    ..
                                } => {
                                    println!("Binlog version: {}", binlog_version);
                                    println!("Server version: {}", mysql_server_version);
                                    println!("Create_timestamp: {}", create_timestamp);
                                }
                                _ => unreachable!(),
                            }
                        }
                        Err(e) => println!("invalid binlog file {:?}", e),
                    }
                }
            }
        },
    }
}
