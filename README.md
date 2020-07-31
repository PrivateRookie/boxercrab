# boxercrab
MySQL binlog parser impl with Rust

![Rust](https://github.com/PrivateRookie/boxercrab/workflows/Rust/badge.svg)
[![BoxerCrab](https://tokei.rs/b1/github/PrivateRookie/boxercrab?category=code)](https://github.com/PrivateRookie/boxercrab)
![Code Coverage](https://github.com/PrivateRookie/boxercrab/workflows/Code%20Coverage/badge.svg)

Boxercrab tried to parse every field in binlog, but for the reasons of documentation, ability, etc., some fields could not be parsed yet.

Parsed events matrix:

| Hex  | Event Name               | Parsed | Note               |
| ---- | ------------------------ | ------ | ------------------ |
| 0x01 | START_EVENT_V3           | N      | too old to support |
| 0x02 | QUERY_EVENT              | Y      |                    |
| 0x03 | STOP_EVENT               | Y      |                    |
| 0x04 | ROTATE_EVENT             | Y      |                    |
| 0x05 | INTVAR_EVENT             | Y      |                    |
| 0x06 | LOAD_EVENT               | Y      | not tested         |
| 0x07 | SLAVE_EVENT              | Y      | not tested         |
| 0x08 | CREATE_FILE_EVENT        | Y      | not tested         |
| 0x09 | APPEND_BLOCK_EVENT       | Y      | not tested         |
| 0x0a | EXEC_LOAD_EVENT          | Y      |                    |
| 0x0b | DELETE_FILE_EVENT        | Y      | not tested         |
| 0x0c | NEW_LOAD_EVENT           | Y      | not tested         |
| 0x0d | RAND_EVENT               | Y      |                    |
| 0x0e | USER_VAR_EVENT           | Y      | not fully tested   |
| 0x0f | FORMAT_DESCRIPTION_EVENT | Y      |                    |
| 0x10 | XID_EVENT                | Y      |                    |
| 0x11 | BEGIN_LOAD_QUERY_EVENT   | Y      |                    |
| 0x12 | EXECUTE_LOAD_QUERY_EVENT | Y      |                    |
| 0x13 | TABLE_MAP_EVENT          | Y      | not fully tested   |
| 0x14 | WRITE_ROWS_EVENTv0       | N      |                    |
| 0x15 | UPDATE_ROWS_EVENTv0      | N      |                    |
| 0x16 | DELETE_ROWS_EVENTv0      | N      |                    |
| 0x17 | WRITE_ROWS_EVENTv1       | N      |                    |
| 0x18 | UPDATE_ROWS_EVENTv1      | N      |                    |
| 0x19 | DELETE_ROWS_EVENTv1      | N      |                    |
| 0x1a | INCIDENT_EVENT           | Y      | not tested         |
| 0x1b | HEARTBEAT_EVENT          | Y      | not tested         |
| 0x1c | IGNORABLE_EVENT          | N      |                    |
| 0x1d | ROWS_QUERY_EVENT         | Y      |                    |
| 0x1e | WRITE_ROWS_EVENTv2       | Y      | not fully tested   |
| 0x1f | UPDATE_ROWS_EVENTv2      | Y      | not fully tested   |
| 0x20 | DELETE_ROWS_EVENTv2      | Y      | not fully tested   |
| 0x21 | GTID_EVENT               | Y      |                    |
| 0x22 | ANONYMOUS_GTID_EVENT     | Y      |                    |
| 0x23 | PREVIOUS_GTIDS_EVENT     | Y      |                    |


Of course, I can't guarantee that the all fields have been parsed correctly. If you encounter an error, please contact me. It is best to attach the binlog file.

## usage

### cli

install cli tool

```bash
cargo install --bin bcrab --git https://github.com/PrivateRookie/boxercrab.git
```

#### all commands

```bash
MySQL binlog tool impl with Rust

USAGE:
    bcrab [FLAGS] <SUBCOMMAND>

FLAGS:
    -d, --debug      enable debug info
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    desc     Show bin log desc msg
    help     Prints this message or the help of the given subcommand(s)
    trans    Transform a binlog file to specified format
```


#### trans

this sub command transform binlog to json or yaml file

```bash
bcrab-trans 0.2.0
Transform a binlog file to specified format

USAGE:
    bcrab trans [OPTIONS] <input> [output]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    Output format [default: Json]  [possible values: Json, Yaml]

ARGS:
    <input>     Binlog file path
    <output>    Output file path, if not present, print to stdout
```

#### desc

show desc info for a binlog

```bash
bcrab-desc 0.2.0
show bin log desc msg

USAGE:
    bcrab desc <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <input>    binlog file path
```

### lib

boxercrab can be use as a library too, but doc is not ready yeah, it's in planning.

