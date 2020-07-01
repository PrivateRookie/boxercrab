use super::{Event, Header};
use crate::utils::{extract_n_string, extract_string, pu32, take_till_term_string};
use nom::{
    bytes::complete::take,
    combinator::map,
    multi::{many0, many_m_n},
    number::complete::{le_u16, le_u32, le_u64, le_u8},
    sequence::tuple,
    IResult,
};

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
    status_vars: Vec<QueryStatusVar>,
    schema: String,
    query: String,
    checksum: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QueryStatusVar {
    Q_FLAGS2_CODE(Q_FLAGS2_CODE_VAL),
    Q_SQL_MODE_CODE(Q_SQL_MODE_CODE_VAL),
    Q_CATALOG(String),
    Q_AUTO_INCREMENT(u16, u16),
    Q_CHARSET_CODE(u16, u16, u16),
    Q_TIME_ZONE_CODE(String),
    Q_CATALOG_NZ_CODE(String),
    Q_LC_TIME_NAMES_CODE(u16),
    Q_CHARSET_DATABASE_CODE(u16),
    Q_TABLE_MAP_FOR_UPDATE_CODE(u64),
    Q_MASTER_DATA_WRITTEN_CODE(u32),
    Q_INVOKERS(String, String),
    Q_UPDATED_DB_NAMES(Vec<String>),
    // NOTE this field take 3 bytes
    Q_MICROSECONDS(u32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Q_FLAGS2_CODE_VAL {
    pub auto_is_null: bool,
    pub auto_commit: bool,
    pub foreign_key_checks: bool,
    pub unique_checks: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Q_SQL_MODE_CODE_VAL {
    real_as_float: bool,
    pipes_as_concat: bool,
    ansi_quotes: bool,
    ignore_space: bool,
    not_used: bool,
    only_full_group_by: bool,
    no_unsigned_subtraction: bool,
    no_dir_in_create: bool,
    postgresql: bool,
    oracle: bool,
    mssql: bool,
    db2: bool,
    maxdb: bool,
    no_key_options: bool,
    no_table_options: bool,
    no_field_options: bool,
    mysql323: bool,
    mysql40: bool,
    ansi: bool,
    no_auto_value_on_zero: bool,
    no_backslash_escapes: bool,
    strict_trans_tables: bool,
    strict_all_tables: bool,
    no_zero_in_date: bool,
    no_zero_date: bool,
    invalid_dates: bool,
    error_for_division_by_zero: bool,
    traditional: bool,
    no_auto_create_user: bool,
    high_not_precedence: bool,
    no_engine_substitution: bool,
    pad_char_to_full_length: bool,
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, slave_proxy_id) = le_u32(input)?;
    let (i, execution_time) = le_u32(i)?;
    let (i, schema_length) = le_u8(i)?;
    let (i, error_code) = le_u16(i)?;
    let (i, status_vars_length) = le_u16(i)?;
    let (i, raw_vars) = take(status_vars_length)(i)?;
    let (remain, status_vars) = many0(parse_status_var)(raw_vars)?;
    assert_eq!(remain.len(), 0);
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

fn parse_status_var<'a>(input: &'a [u8]) -> IResult<&'a [u8], QueryStatusVar> {
    let (i, key) = le_u8(input)?;
    match key {
        0x00 => {
            let (i, code) = le_u32(i)?;
            let auto_is_null = (code >> 14) % 2 == 1;
            let auto_commit = (code >> 19) % 2 == 0;
            let foreign_key_checks = (code >> 26) % 2 == 0;
            let unique_checks = (code >> 17) % 2 == 0;
            Ok((
                i,
                QueryStatusVar::Q_FLAGS2_CODE(Q_FLAGS2_CODE_VAL {
                    auto_is_null,
                    auto_commit,
                    foreign_key_checks,
                    unique_checks,
                }),
            ))
        }
        0x01 => {
            let (i, code) = le_u64(i)?;
            let val = Q_SQL_MODE_CODE_VAL {
                real_as_float: (code >> 0) % 2 == 1,
                pipes_as_concat: (code >> 1) % 2 == 1,
                ansi_quotes: (code >> 2) % 2 == 1,
                ignore_space: (code >> 3) % 2 == 1,
                not_used: (code >> 4) % 2 == 1,
                only_full_group_by: (code >> 5) % 2 == 1,
                no_unsigned_subtraction: (code >> 6) % 2 == 1,
                no_dir_in_create: (code >> 7) % 2 == 1,
                postgresql: (code >> 8) % 2 == 1,
                oracle: (code >> 9) % 2 == 1,
                mssql: (code >> 10) % 2 == 1,
                db2: (code >> 11) % 2 == 1,
                maxdb: (code >> 12) % 2 == 1,
                no_key_options: (code >> 13) % 2 == 1,
                no_table_options: (code >> 14) % 2 == 1,
                no_field_options: (code >> 15) % 2 == 1,
                mysql323: (code >> 16) % 2 == 1,
                mysql40: (code >> 17) % 2 == 1,
                ansi: (code >> 18) % 2 == 1,
                no_auto_value_on_zero: (code >> 19) % 2 == 1,
                no_backslash_escapes: (code >> 20) % 2 == 1,
                strict_trans_tables: (code >> 21) % 2 == 1,
                strict_all_tables: (code >> 22) % 2 == 1,
                no_zero_in_date: (code >> 23) % 2 == 1,
                no_zero_date: (code >> 24) % 2 == 1,
                invalid_dates: (code >> 25) % 2 == 1,
                error_for_division_by_zero: (code >> 26) % 2 == 1,
                traditional: (code >> 27) % 2 == 1,
                no_auto_create_user: (code >> 28) % 2 == 1,
                high_not_precedence: (code >> 29) % 2 == 1,
                no_engine_substitution: (code >> 30) % 2 == 1,
                pad_char_to_full_length: (code >> 31) % 2 == 1,
            };
            Ok((i, QueryStatusVar::Q_SQL_MODE_CODE(val)))
        }
        0x02 => {
            let (i, len) = le_u8(i)?;
            let (i, val) = map(take(len), |s: &[u8]| extract_n_string(s, len as usize))(i)?;
            let (i, term) = le_u8(i)?;
            assert_eq!(term, 0x00);
            Ok((i, QueryStatusVar::Q_CATALOG(val)))
        }
        0x03 => {
            let (i, incr) = le_u16(i)?;
            let (i, offset) = le_u16(i)?;
            Ok((i, QueryStatusVar::Q_AUTO_INCREMENT(incr, offset)))
        }
        0x04 => {
            let (i, (client, conn, server)) = tuple((le_u16, le_u16, le_u16))(i)?;
            Ok((i, QueryStatusVar::Q_CHARSET_CODE(client, conn, server)))
        }
        0x05 => {
            let (i, len) = le_u8(i)?;
            let (i, tz) = map(take(len), |s: &[u8]| extract_string(s))(i)?;
            Ok((i, QueryStatusVar::Q_TIME_ZONE_CODE(tz)))
        }
        0x06 => {
            let (i, len) = le_u8(i)?;
            let (i, val) = map(take(len), |s: &[u8]| extract_string(s))(i)?;
            Ok((i, QueryStatusVar::Q_CATALOG_NZ_CODE(val)))
        }
        0x07 => map(le_u16, |v| QueryStatusVar::Q_LC_TIME_NAMES_CODE(v))(i),
        0x08 => map(le_u16, |v| QueryStatusVar::Q_CHARSET_DATABASE_CODE(v))(i),
        0x09 => map(le_u64, |v| QueryStatusVar::Q_TABLE_MAP_FOR_UPDATE_CODE(v))(i),
        0x0a => map(le_u32, |v| QueryStatusVar::Q_MASTER_DATA_WRITTEN_CODE(v))(i),
        0x0b => {
            let (i, len) = le_u8(i)?;
            let (i, user) = map(take(len), |s: &[u8]| extract_n_string(s, len as usize))(i)?;
            let (i, len) = le_u8(i)?;
            let (i, host) = map(take(len), |s: &[u8]| extract_n_string(s, len as usize))(i)?;
            Ok((i, QueryStatusVar::Q_INVOKERS(user, host)))
        }
        0x0c => {
            let (i, count) = le_u8(i)?;
            let (i, val) = many_m_n(count as usize, count as usize, take_till_term_string)(i)?;
            Ok((i, QueryStatusVar::Q_UPDATED_DB_NAMES(val)))
        }
        0x0d => map(pu32, |val| QueryStatusVar::Q_MICROSECONDS(val))(i),
        __ => unreachable!(),
    }
}
