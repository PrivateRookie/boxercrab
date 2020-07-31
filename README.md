# boxercrab
MySQL binlog parser impl with Rust

![Rust](https://github.com/PrivateRookie/boxercrab/workflows/Rust/badge.svg)
[![BoxerCrab](https://tokei.rs/b1/github/PrivateRookie/boxercrab?category=code)](https://github.com/PrivateRookie/boxercrab)
![Code Coverage](https://github.com/PrivateRookie/boxercrab/workflows/Code%20Coverage/badge.svg)

Boxercrab tried to parse every field in binlog, but for the reasons of documentation, ability, etc., some fields could not be parsed yet. Missing fields are

- rows fields in row events, These fields contain the specific data of the row event, but due to the complex field types, there is currently no resolution. But it is already planned :)
- gtid_sets field in previous gtid event

Of course, I can't guarantee that the other fields have been parsed correctly. If you find a program error, please contact me. It is best to attach the parsed binlog file.

## usage

### install

```bash
cargo install --bin bcrab --git https://github.com/PrivateRookie/boxercrab.git
```

### serde

this sub command transform binlog to json or yaml file

```bash
bcrab-serde 0.1.0
transform a binlog file to specified format

USAGE:
    bcrab serde [OPTIONS] <input> <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    output format [default: Json]  [possible values: Json, Yaml]

ARGS:
    <input>     binlog file path
    <output>    output file path
```

### desc

show desc info for a binlog

```bash
bcrab-desc 0.1.0
show bin log desc msg

USAGE:
    bcrab desc <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <input>    binlog file path
```
