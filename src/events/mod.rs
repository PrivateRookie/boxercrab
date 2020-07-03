use crate::utils::{extract_string, parse_lenenc_int};
use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    multi::many0,
    number::complete::{le_i64, le_u16, le_u32, le_u64, le_u8},
    IResult,
};

mod query;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    pub timestamp: u32,
    pub event_type: u8,
    pub server_id: u32,
    pub event_size: u32,
    pub log_pos: u32,
    pub flags: u16,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (i, timestamp) = le_u32(input)?;
    let (i, event_type) = le_u8(i)?;
    let (i, server_id) = le_u32(i)?;
    let (i, event_size) = le_u32(i)?;
    let (i, log_pos) = le_u32(i)?;
    let (i, flags) = le_u16(i)?;
    Ok((
        i,
        Header {
            timestamp,
            event_type,
            server_id,
            event_size,
            log_pos,
            flags,
        },
    ))
}

pub fn check_start(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag([254, 98, 105, 110])(i)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    // doc: https://dev.mysql.com/doc/internals/en/query-event.html
    // source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/statement_events.h#L44-L426
    // layout: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/statement_events.h#L627-L643
    Query {
        header: Header,
        slave_proxy_id: u32, // thread_id
        execution_time: u32,
        schema_length: u8, // length of current select schema name
        error_code: u16,
        status_vars_length: u16,
        status_vars: Vec<query::QueryStatusVar>,
        schema: String,
        query: String,
        checksum: u32,
    },
    // source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L295-L344
    // event_data layout: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L387-L416
    FormatDesc {
        header: Header,
        binlog_version: u16,
        mysql_server_version: String,
        create_timestamp: u32,
        event_header_length: u8,
        supported_types: Vec<u8>,
        checksum_alg: u8,
        checksum: u32,
    },
    // source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L932-L991
    AnonymousGtid {
        header: Header,
        rbr_only: bool,
        encoded_sig_length: u32,
        encoded_gno_length: u32,
        // FIXME unknown fields
        unknown: Vec<u8>,
        last_committed: i64,
        sequence_number: i64,
        checksum: u32,
    },
    // source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L1073-L1103
    PreviousGtids {
        header: Header,
        // FIXME this field may be wrong
        gtid_sets: Vec<u8>,
        buf_size: u32,
        checksum: u32,
    },
    TableMap {
        header: Header,
        // table_id take 6 bytes in buffer
        table_id: u64,
        flags: u16,
        schema_length: u8,
        schema: String,
        // [00] term sign in layout
        table_name_length: u8,
        table_name: String,
        // [00] term sign in layout
        // len encoded integer
        column_count: u64,
        column_type_def: Vec<u8>,
        // len encoded string
        column_meta_def: Vec<u8>,
        null_bit_mask: Vec<u8>,
        checksum: u32,
    },
}

impl Event {
    pub fn parse<'a>(input: &'a [u8]) -> IResult<&'a [u8], Event> {
        let (input, header) = parse_header(input)?;
        match header.event_type {
            0x02 => parse_query(input, header),
            0x0f => parse_format_desc(input, header),
            0x13 => parse_table_map(input, header),
            0x22 => parse_anonymous_gtid(input, header),
            0x23 => parse_previous_gtids(input, header),
            _ => unreachable!(),
        }
    }
}

fn pu64(input: &[u8]) -> IResult<&[u8], u64> {
    le_u64(input)
}

fn parse_format_desc<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, binlog_version) = le_u16(input)?;
    let (i, mysql_server_version) = map(take(50usize), |s: &[u8]| extract_string(s))(i)?;
    let (i, create_timestamp) = le_u32(i)?;
    let (i, event_header_length) = le_u8(i)?;
    let num = header.event_size - 19 - (2 + 50 + 4 + 1) - 1 - 4;
    let (i, supported_types) = map(take(num), |s: &[u8]| s.to_vec())(i)?;
    let (i, checksum_alg) = le_u8(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::FormatDesc {
            header,
            binlog_version,
            mysql_server_version,
            create_timestamp,
            event_header_length,
            supported_types,
            checksum_alg,
            checksum,
        },
    ))
}

fn parse_anonymous_gtid<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, rbr_only) = map(le_u8, |t: u8| t == 0)(input)?;
    let (i, encoded_sig_length) = le_u32(i)?;
    let (i, encoded_gno_length) = le_u32(i)?;
    let (i, unknown) = map(
        take(header.event_size - 19 - (1 + 4 * 2 + 8 * 2 + 4)),
        |s: &[u8]| s.to_vec(),
    )(i)?;
    let (i, last_committed) = le_i64(i)?;
    let (i, sequence_number) = le_i64(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::AnonymousGtid {
            header,
            rbr_only,
            encoded_sig_length,
            encoded_gno_length,
            last_committed,
            sequence_number,
            unknown,
            checksum,
        },
    ))
}

fn parse_previous_gtids<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, gtid_sets) = map(take(header.event_size - 19 - 4 - 4), |s: &[u8]| s.to_vec())(input)?;
    let (i, buf_size) = le_u32(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::PreviousGtids {
            header,
            gtid_sets,
            buf_size,
            checksum,
        },
    ))
}

fn parse_table_map<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, table_id): (&'a [u8], u64) = map(take(6usize), |id_raw: &[u8]| {
        let mut filled = id_raw.to_vec();
        filled.extend(vec![0, 0]);
        pu64(&filled).unwrap().1
    })(input)?;
    let (i, flags) = le_u16(i)?;
    let (i, schema_length) = le_u8(i)?;
    let (i, schema) = map(take(schema_length), |s: &[u8]| extract_string(s))(i)?;
    let (i, term) = le_u8(i)?;
    assert_eq!(term, 0);
    let (i, table_name_length) = le_u8(i)?;
    let (i, table_name) = map(take(table_name_length), |s: &[u8]| extract_string(s))(i)?;
    let (i, term) = le_u8(i)?;
    assert_eq!(term, 0);
    let (i, column_count) = parse_lenenc_int(i)?;
    let (i, column_type_def) = map(take(column_count), |s: &[u8]| s.to_vec())(i)?;
    let (i, column_meta_count) = parse_lenenc_int(i)?;
    let (i, column_meta_def) = map(take(column_meta_count), |s: &[u8]| s.to_vec())(i)?;
    let mask_len = (column_count + 8) / 7;
    let (i, null_bit_mask) = map(take(mask_len), |s: &[u8]| s.to_vec())(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::TableMap {
            header,
            table_id,
            flags,
            schema_length,
            schema,
            table_name_length,
            table_name,
            column_count,
            column_type_def,
            column_meta_def,
            null_bit_mask,
            checksum,
        },
    ))
}

fn parse_query<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, slave_proxy_id) = le_u32(input)?;
    let (i, execution_time) = le_u32(i)?;
    let (i, schema_length) = le_u8(i)?;
    let (i, error_code) = le_u16(i)?;
    let (i, status_vars_length) = le_u16(i)?;
    let (i, raw_vars) = take(status_vars_length)(i)?;
    let (remain, status_vars) = many0(query::parse_status_var)(raw_vars)?;
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
        Event::Query {
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
        },
    ))
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_anonymous_gtids() {
        use super::parse_header;
        let input: Vec<u8> = vec![
            54, 157, 253, 94, 34, 123, 0, 0, 0, 65, 0, 0, 0, 219, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 10, 21, 198, 18,
        ];
        let (i, header) = parse_header(&input).unwrap();
        let (i, event) = parse_anonymous_gtid(i, header).unwrap();
        match event {
            Event::AnonymousGtid {
                last_committed,
                sequence_number,
                rbr_only,
                ..
            } => {
                assert_eq!(last_committed, 0);
                assert_eq!(sequence_number, 1);
                assert_eq!(rbr_only, false);
                assert_eq!(i.len(), 0);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_format_desc() {
        use super::parse_header;
        let input: Vec<u8> = vec![
            220, 156, 253, 94, 15, 123, 0, 0, 0, 119, 0, 0, 0, 123, 0, 0, 0, 1, 0, 4, 0, 53, 46,
            55, 46, 50, 57, 45, 108, 111, 103, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 156, 253, 94,
            19, 56, 13, 0, 8, 0, 18, 0, 4, 4, 4, 4, 18, 0, 0, 95, 0, 4, 26, 8, 0, 0, 0, 8, 8, 8, 2,
            0, 0, 0, 10, 10, 10, 42, 42, 0, 18, 52, 0, 1, 207, 88, 126, 238,
        ];
        let (i, header) = parse_header(&input).unwrap();
        let (i, event) = parse_format_desc(i, header).unwrap();
        match event {
            Event::FormatDesc {
                binlog_version,
                mysql_server_version,
                create_timestamp,
                ..
            } => {
                assert_eq!(binlog_version, 4);
                assert_eq!(mysql_server_version, "5.7.29-log");
                assert_eq!(create_timestamp, 1593679068);
                assert_eq!(i.len(), 0);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_previous_gtids() {
        use super::parse_header;

        let input: Vec<u8> = vec![
            220, 156, 253, 94, 35, 123, 0, 0, 0, 31, 0, 0, 0, 154, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 82, 75, 196, 253,
        ];
        let (i, header) = parse_header(&input).unwrap();
        let (i, _) = parse_previous_gtids(i, header).unwrap();
        assert_eq!(i.len(), 0);
        // TODO do more parse
    }

    #[test]
    fn test_table_map() {
        use super::parse_header;

        let input: Vec<u8> = vec![
            170, 157, 253, 94, 19, 123, 0, 0, 0, 60, 0, 0, 0, 246, 2, 0, 0, 0, 0, 109, 0, 0, 0, 0,
            0, 1, 0, 4, 116, 101, 115, 116, 0, 10, 114, 117, 110, 111, 111, 98, 95, 116, 98, 108,
            0, 4, 3, 15, 15, 10, 4, 44, 1, 120, 0, 8, 194, 168, 53, 68,
        ];
        let (i, header) = parse_header(&input).unwrap();
        let (i, event) = parse_table_map(i, header).unwrap();
        match event {
            Event::TableMap {
                table_id, schema, ..
            } => {
                assert_eq!(i.len(), 0);
                // TODO do more checks here
                assert_eq!(table_id, 109);
                assert_eq!(schema, "test".to_string());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_query() {
        use super::parse_header;

        let input: Vec<u8> = vec![
            54, 157, 253, 94, 2, 123, 0, 0, 0, 78, 1, 0, 0, 41, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
            0, 4, 0, 0, 33, 0, 0, 0, 0, 0, 0, 1, 32, 0, 160, 85, 0, 0, 0, 0, 6, 3, 115, 116, 100,
            4, 33, 0, 33, 0, 224, 0, 12, 1, 116, 101, 115, 116, 0, 116, 101, 115, 116, 0, 67, 82,
            69, 65, 84, 69, 32, 84, 65, 66, 76, 69, 32, 73, 70, 32, 78, 79, 84, 32, 69, 88, 73, 83,
            84, 83, 32, 96, 114, 117, 110, 111, 111, 98, 95, 116, 98, 108, 96, 40, 10, 32, 32, 32,
            96, 114, 117, 110, 111, 111, 98, 95, 105, 100, 96, 32, 73, 78, 84, 32, 85, 78, 83, 73,
            71, 78, 69, 68, 32, 65, 85, 84, 79, 95, 73, 78, 67, 82, 69, 77, 69, 78, 84, 44, 10, 32,
            32, 32, 96, 114, 117, 110, 111, 111, 98, 95, 116, 105, 116, 108, 101, 96, 32, 86, 65,
            82, 67, 72, 65, 82, 40, 49, 48, 48, 41, 32, 78, 79, 84, 32, 78, 85, 76, 76, 44, 10, 32,
            32, 32, 96, 114, 117, 110, 111, 111, 98, 95, 97, 117, 116, 104, 111, 114, 96, 32, 86,
            65, 82, 67, 72, 65, 82, 40, 52, 48, 41, 32, 78, 79, 84, 32, 78, 85, 76, 76, 44, 10, 32,
            32, 32, 96, 115, 117, 98, 109, 105, 115, 115, 105, 111, 110, 95, 100, 97, 116, 101, 96,
            32, 68, 65, 84, 69, 44, 10, 32, 32, 32, 80, 82, 73, 77, 65, 82, 89, 32, 75, 69, 89, 32,
            40, 32, 96, 114, 117, 110, 111, 111, 98, 95, 105, 100, 96, 32, 41, 10, 41, 69, 78, 71,
            73, 78, 69, 61, 73, 110, 110, 111, 68, 66, 32, 68, 69, 70, 65, 85, 76, 84, 32, 67, 72,
            65, 82, 83, 69, 84, 61, 117, 116, 102, 56, 120, 116, 234, 84,
        ];
        let (i, header) = parse_header(&input).unwrap();
        let (i, event) = parse_query(i, header.clone()).unwrap();
        assert_eq!(i.len(), 0);
        assert_eq!(
        event,
        Event::Query {
            header,
            slave_proxy_id: 3,
            execution_time: 0,
            schema_length: 4,
            schema: String::from("test"),
            error_code: 0,
            status_vars_length: 33,
            status_vars: vec![
                query::QueryStatusVar::Q_FLAGS2_CODE(query::Q_FLAGS2_CODE_VAL {
                    auto_is_null: false,
                    auto_commit: true,
                    foreign_key_checks: true,
                    unique_checks: true,
                }),
                query::QueryStatusVar::Q_SQL_MODE_CODE(query::Q_SQL_MODE_CODE_VAL {
                    real_as_float: false,
                    pipes_as_concat: false,
                    ansi_quotes: false,
                    ignore_space: false,
                    not_used: false,
                    only_full_group_by: true,
                    no_unsigned_subtraction: false,
                    no_dir_in_create: false,
                    postgresql: false,
                    oracle: false,
                    mssql: false,
                    db2: false,
                    maxdb: false,
                    no_key_options: false,
                    no_table_options: false,
                    no_field_options: false,
                    mysql323: false,
                    mysql40: false,
                    ansi: false,
                    no_auto_value_on_zero: false,
                    no_backslash_escapes: false,
                    strict_trans_tables: true,
                    strict_all_tables: false,
                    no_zero_in_date: true,
                    no_zero_date: true,
                    invalid_dates: false,
                    error_for_division_by_zero: true,
                    traditional: false,
                    no_auto_create_user: true,
                    high_not_precedence: false,
                    no_engine_substitution: true,
                    pad_char_to_full_length: false
                }),
                query::QueryStatusVar::Q_CATALOG_NZ_CODE("std".to_string()),
                query::QueryStatusVar::Q_CHARSET_CODE(33, 33, 224),
                query::QueryStatusVar::Q_UPDATED_DB_NAMES(vec!["test".to_string()])
            ],
            query: String::from("CREATE TABLE IF NOT EXISTS `runoob_tbl`(\n   `runoob_id` INT UNSIGNED AUTO_INCREMENT,\n   `runoob_title` VARCHAR(100) NOT NULL,\n   `runoob_author` VARCHAR(40) NOT NULL,\n   `submission_date` DATE,\n   PRIMARY KEY ( `runoob_id` )\n)ENGINE=InnoDB DEFAULT CHARSET=utf8"),
            checksum: 1424651384,
        }
    );
    }
}
