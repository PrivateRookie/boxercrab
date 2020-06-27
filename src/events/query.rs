use super::{extract_string, Event, Header};
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_i32, le_u16, le_u32, le_u8},
    IResult,
};
use num_enum::TryFromPrimitive;

// doc: https://dev.mysql.com/doc/internals/en/query-event.html
// source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/statement_events.h#L44-L426
// layout: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/statement_events.h#L627-L643
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Query {
    header: Header,
    slave_proxy_id: u32, // thread_id
    execution_time: u32,
    schema_length: u8, // length of current select schema name
    error_code: u16,
    status_vars_length: u16,
    status_vars: Vec<u8>,
    schema: String,
    query: String,
    checksum: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QueryStatusFlags {
    Q_FLAGS2_CODE = 0x00,
    Q_SQL_MODE_CODE = 0x01,
    Q_CATALOG = 0x02,
    Q_AUTO_INCREMENT = 0x03,
    Q_CHARSET_CODE = 0x04,
    Q_TIME_ZONE_CODE = 0x05,
    Q_CATALOG_NZ_CODE = 0x06,
    Q_LC_TIME_NAMES_CODE = 0x07,
    Q_CHARSET_DATABASE_CODE = 0x08,
    Q_TABLE_MAP_FOR_UPDATE_CODE = 0x09,
    Q_MASTER_DATA_WRITTEN_CODE = 0x0a,
    Q_INVOKERS = 0x0b,
    Q_UPDATED_DB_NAMES = 0x0c,
    Q_MICROSECONDS = 0x0d,
}

#[derive(Debug, PartialEq, Eq, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum Q_FLAGS2_CODE_VAL {
    OPTION_AUTO_IS_NULL = 0x00004000,
    OPTION_NOT_AUTOCOMMIT = 0x00080000,
    OPTION_NO_FOREIGN_KEY_CHECKS = 0x04000000,
    OPTION_RELAXED_UNIQUE_CHECKS = 0x08000000,
}

#[derive(Debug, PartialEq, Eq, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum Q_SQL_MODE_CODE {
    MODE_REAL_AS_FLOAT = 0x00000001,
    MODE_PIPES_AS_CONCAT = 0x00000002,
    MODE_ANSI_QUOTES = 0x00000004,
    MODE_IGNORE_SPACE = 0x00000008,
    MODE_NOT_USED = 0x00000010,
    MODE_ONLY_FULL_GROUP_BY = 0x00000020,
    MODE_NO_UNSIGNED_SUBTRACTION = 0x00000040,
    MODE_NO_DIR_IN_CREATE = 0x00000080,
    MODE_POSTGRESQL = 0x00000100,
    MODE_ORACLE = 0x00000200,
    MODE_MSSQL = 0x00000400,
    MODE_DB2 = 0x00000800,
    MODE_MAXDB = 0x00001000,
    MODE_NO_KEY_OPTIONS = 0x00002000,
    MODE_NO_TABLE_OPTIONS = 0x00004000,
    MODE_NO_FIELD_OPTIONS = 0x00008000,
    MODE_MYSQL323 = 0x00010000,
    MODE_MYSQL40 = 0x00020000,
    MODE_ANSI = 0x00040000,
    MODE_NO_AUTO_VALUE_ON_ZERO = 0x00080000,
    MODE_NO_BACKSLASH_ESCAPES = 0x00100000,
    MODE_STRICT_TRANS_TABLES = 0x00200000,
    MODE_STRICT_ALL_TABLES = 0x00400000,
    MODE_NO_ZERO_IN_DATE = 0x00800000,
    MODE_NO_ZERO_DATE = 0x01000000,
    MODE_INVALID_DATES = 0x02000000,
    MODE_ERROR_FOR_DIVISION_BY_ZERO = 0x04000000,
    MODE_TRADITIONAL = 0x08000000,
    MODE_NO_AUTO_CREATE_USER = 0x10000000,
    MODE_HIGH_NOT_PRECEDENCE = 0x20000000,
    MODE_NO_ENGINE_SUBSTITUTION = 0x40000000,
    MODE_PAD_CHAR_TO_FULL_LENGTH = 0x80000000,
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    println!("{:?}", &header);
    let (i, slave_proxy_id) = le_u32(input)?;
    let (i, execution_time) = le_u32(i)?;
    let (i, schema_length) = le_u8(i)?;
    let (i, error_code) = le_u16(i)?;
    let (i, status_vars_length) = le_u16(i)?;
    // TODO parse status_vars
    let (i, status_vars) = map(take(status_vars_length), |s: &[u8]| s.to_vec())(i)?;
    let (i, schema) = map(take(schema_length), |s: &[u8]| {
        String::from_utf8(s[0..schema_length as usize].to_vec()).unwrap()
    })(i)?;
    let (i, _) = take(1usize)(i)?;
    let (i, query) = map(
        take(
            header.event_size
                - 19
                - 4
                - 4
                - 1
                - 2
                - 2
                - status_vars_length as u32
                - schema_length as u32
                - 1
                - 4,
        ),
        |s: &[u8]| extract_string(s),
    )(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::Query(Query {
            header,
            slave_proxy_id,
            execution_time,
            schema_length,
            error_code,
            status_vars_length,
            status_vars,
            schema,
            query,
            checksum,
        }),
    ))
}
