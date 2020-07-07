use crate::utils::{string_var, extract_string, pu32, string_nul};
use nom::{
    bytes::complete::take,
    combinator::map,
    multi::many_m_n,
    number::complete::{le_u16, le_u32, le_u64, le_u8},
    sequence::tuple,
    IResult,
};

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
    pub real_as_float: bool,
    pub pipes_as_concat: bool,
    pub ansi_quotes: bool,
    pub ignore_space: bool,
    pub not_used: bool,
    pub only_full_group_by: bool,
    pub no_unsigned_subtraction: bool,
    pub no_dir_in_create: bool,
    pub postgresql: bool,
    pub oracle: bool,
    pub mssql: bool,
    pub db2: bool,
    pub maxdb: bool,
    pub no_key_options: bool,
    pub no_table_options: bool,
    pub no_field_options: bool,
    pub mysql323: bool,
    pub mysql40: bool,
    pub ansi: bool,
    pub no_auto_value_on_zero: bool,
    pub no_backslash_escapes: bool,
    pub strict_trans_tables: bool,
    pub strict_all_tables: bool,
    pub no_zero_in_date: bool,
    pub no_zero_date: bool,
    pub invalid_dates: bool,
    pub error_for_division_by_zero: bool,
    pub traditional: bool,
    pub no_auto_create_user: bool,
    pub high_not_precedence: bool,
    pub no_engine_substitution: bool,
    pub pad_char_to_full_length: bool,
}

pub fn parse_status_var<'a>(input: &'a [u8]) -> IResult<&'a [u8], QueryStatusVar> {
    let (i, key) = le_u8(input)?;
    match key {
        0x00 => {
            let (i, code) = le_u32(i)?;
            let auto_is_null = (code >> 14) % 2 == 1;
            let auto_commit = (code >> 19) % 2 == 0;
            let foreign_key_checks = (code >> 26) % 2 == 0;
            let unique_checks = (code >> 27) % 2 == 0;
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
            let (i, val) = map(take(len), |s: &[u8]| string_var(s, len as usize))(i)?;
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
            let (i, user) = map(take(len), |s: &[u8]| string_var(s, len as usize))(i)?;
            let (i, len) = le_u8(i)?;
            let (i, host) = map(take(len), |s: &[u8]| string_var(s, len as usize))(i)?;
            Ok((i, QueryStatusVar::Q_INVOKERS(user, host)))
        }
        0x0c => {
            let (i, count) = le_u8(i)?;
            let (i, val) = many_m_n(count as usize, count as usize, string_nul)(i)?;
            Ok((i, QueryStatusVar::Q_UPDATED_DB_NAMES(val)))
        }
        0x0d => map(pu32, |val| QueryStatusVar::Q_MICROSECONDS(val))(i),
        __ => unreachable!(),
    }
}
