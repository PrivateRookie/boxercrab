use crate::{
    mysql::{ColTypes, ColValues},
    utils::{extract_string, int_lenenc, pu64, string_fixed, string_nul, string_var},
};
use lazy_static::lazy_static;
use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    multi::{many0, many1, many_m_n},
    number::complete::{le_i64, le_u16, le_u32, le_u64, le_u8},
    sequence::tuple,
    IResult,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod query;
mod rows;

lazy_static! {
    static ref TABLE_MAP: Arc<Mutex<HashMap<u64, Vec<ColTypes>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct EventFlag {
    in_use: bool,
    forced_rotate: bool,
    thread_specific: bool,
    suppress_use: bool,
    update_table_map_version: bool,
    artificial: bool,
    relay_log: bool,
    ignorable: bool,
    no_filter: bool,
    mts_isolate: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct Header {
    pub timestamp: u32,
    pub event_type: u8,
    pub server_id: u32,
    pub event_size: u32,
    pub log_pos: u32,
    pub flags: EventFlag,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (i, timestamp) = le_u32(input)?;
    let (i, event_type) = le_u8(i)?;
    let (i, server_id) = le_u32(i)?;
    let (i, event_size) = le_u32(i)?;
    let (i, log_pos) = le_u32(i)?;
    let (i, flags) = map(le_u16, |f: u16| EventFlag {
        in_use: (f >> 0) % 2 == 1,
        forced_rotate: (f >> 1) % 2 == 1,
        thread_specific: (f >> 2) % 2 == 1,
        suppress_use: (f >> 3) % 2 == 1,
        update_table_map_version: (f >> 4) % 2 == 1,
        artificial: (f >> 5) % 2 == 1,
        relay_log: (f >> 6) % 2 == 1,
        ignorable: (f >> 7) % 2 == 1,
        no_filter: (f >> 8) % 2 == 1,
        mts_isolate: (f >> 9) % 2 == 1,
    })(i)?;
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

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum Event {
    // ref: https://dev.mysql.com/doc/internals/en/ignored-events.html#unknown-event
    Unknown {
        header: Header,
        checksum: u32,
    },
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
    // ref: https://dev.mysql.com/doc/internals/en/stop-event.html
    Stop {
        header: Header,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/rotate-event.html
    Rotate {
        header: Header,
        position: u64,
        next_binlog: String,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/intvar-event.html
    IntVar {
        header: Header,
        e_type: IntVarEventType,
        value: u64,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/load-event.html
    Load {
        header: Header,
        thread_id: u32,
        execution_time: u32,
        skip_lines: u32,
        table_name_length: u8,
        schema_length: u8,
        num_fields: u32,
        field_term: u8,
        enclosed_by: u8,
        line_term: u8,
        line_start: u8,
        escaped_by: u8,
        opt_flags: OptFlags,
        empty_flags: EmptyFlags,
        field_name_lengths: Vec<u8>,
        field_names: Vec<String>,
        table_name: String,
        schema_name: String,
        file_name: String,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/ignored-events.html#slave-event
    Slave {
        header: Header,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/create-file-event.html
    CreateFile {
        header: Header,
        file_id: u32,
        block_data: String,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/append-block-event.html
    AppendFile {
        header: Header,
        file_id: u32,
        block_data: String,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/exec-load-event.html
    ExecLoad {
        header: Header,
        file_id: u16,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/delete-file-event.html
    DeleteFile {
        header: Header,
        file_id: u16,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/new-load-event.html
    NewLoad {
        header: Header,
        thread_id: u32,
        execution_time: u32,
        skip_lines: u32,
        table_name_length: u8,
        schema_length: u8,
        num_fields: u32,

        field_term_length: u8,
        field_term: String,
        enclosed_by_length: u8,
        enclosed_by: String,
        line_term_length: u8,
        line_term: String,
        line_start_length: u8,
        line_start: String,
        escaped_by_length: u8,
        escaped_by: String,
        opt_flags: OptFlags,

        field_name_lengths: Vec<u8>,
        field_names: Vec<String>,
        table_name: String,
        schema_name: String,
        file_name: String,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/rand-event.html
    Rand {
        header: Header,
        seed1: u64,
        seed2: u64,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/user-var-event.html
    // source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/statement_events.h#L712-L779
    // NOTE ref is broken !!!
    UserVar {
        header: Header,
        name_length: u32,
        name: String,
        is_null: bool,
        d_type: Option<u8>,
        charset: Option<u32>,
        value_length: Option<u32>,
        value: Option<Vec<u8>>,
        flags: Option<u8>,
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
    XID {
        header: Header,
        xid: u64,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/begin-load-query-event.html
    BeginLoadQuery {
        header: Header,
        file_id: u32,
        block_data: String,
        checksum: u32,
    },
    ExecuteLoadQueryEvent {
        header: Header,
        thread_id: u32,
        execution_time: u32,
        schema_length: u8,
        error_code: u16,
        status_vars_length: u16,
        file_id: u32,
        start_pos: u32,
        end_pos: u32,
        dup_handling_flags: DupHandlingFlags,
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
        columns_type: Vec<ColTypes>,
        null_bits: Vec<u8>,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/incident-event.html
    Incident {
        header: Header,
        d_type: IncidentEventType,
        message_length: u8,
        message: String,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/heartbeat-event.html
    Heartbeat {
        header: Header,
        checksum: u32,
    },
    // ref: https://dev.mysql.com/doc/internals/en/rows-query-event.html
    RowQuery {
        header: Header,
        length: u8,
        query_text: String,
        checksum: u32,
    },
    // https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L1048-L1056
    Gtid {
        header: Header,
        rbr_only: bool,
        source_id: String,
        transaction_id: String,
        ts_type: u8,
        last_committed: i64,
        sequence_number: i64,
        checksum: u32,
    },
    AnonymousGtid {
        header: Header,
        rbr_only: bool,
        source_id: String,
        transaction_id: String,
        ts_type: u8,
        last_committed: i64,
        sequence_number: i64,
        checksum: u32,
    },
    // source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/control_events.h#L1073-L1103
    PreviousGtids {
        header: Header,
        // TODO do more specify parse
        gtid_sets: Vec<u8>,
        buf_size: u32,
        checksum: u32,
    },
    // source https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/rows_event.h#L488-L613
    WriteRowsV2 {
        header: Header,
        // table_id take 6 bytes in buffer
        table_id: u64,
        flags: rows::Flags,
        extra_data_len: u16,
        extra_data: Vec<rows::ExtraData>,
        column_count: u64,
        inserted_image_bits: Vec<u8>,
        rows: Vec<Vec<ColValues>>,
        checksum: u32,
    },
    UpdateRowsV2 {
        header: Header,
        // table_id take 6 bytes in buffer
        table_id: u64,
        flags: rows::Flags,
        extra_data_len: u16,
        extra_data: Vec<rows::ExtraData>,
        column_count: u64,
        before_image_bits: Vec<u8>,
        after_image_bits: Vec<u8>,
        rows: Vec<Vec<ColValues>>,
        checksum: u32,
    },
    DeleteRowsV2 {
        header: Header,
        // table_id take 6 bytes in buffer
        table_id: u64,
        flags: rows::Flags,
        extra_data_len: u16,
        extra_data: Vec<rows::ExtraData>,
        column_count: u64,
        deleted_image_bits: Vec<u8>,
        rows: Vec<Vec<ColValues>>,
        checksum: u32,
    },
}

impl Event {
    pub fn parse<'a>(input: &'a [u8]) -> IResult<&'a [u8], Event> {
        let (input, header) = parse_header(input)?;
        match header.event_type {
            0x00 => parse_unknown(input, header),
            0x02 => parse_query(input, header),
            0x03 => parse_stop(input, header),
            0x04 => parse_rotate(input, header),
            0x05 => parse_intvar(input, header),
            0x06 => parse_load(input, header),
            0x07 => parse_slave(input, header),
            0x08 => parse_create_file(input, header),
            0x09 => parse_append_file(input, header),
            0x0a => parse_exec_load(input, header),
            0x0b => parse_delete_file(input, header),
            0x0c => parse_new_load(input, header),
            0x0d => parse_rand(input, header),
            0x0e => parse_user_var(input, header),
            0x0f => parse_format_desc(input, header),
            0x10 => parse_xid(input, header),
            0x11 => parse_begin_load_query(input, header),
            0x12 => parse_execute_load_query(input, header),
            0x13 => parse_table_map(input, header),
            0x1a => parse_incident(input, header),
            0x1b => parse_heartbeat(input, header),
            0x1d => parse_row_query(input, header),
            0x14..=0x19 => unreachable!(),
            0x1e => parse_write_rows_v2(input, header),
            0x1f => parse_update_rows_v2(input, header),
            0x20 => parse_delete_rows_v2(input, header),
            0x21 => parse_gtid(input, header),
            0x22 => parse_anonymous_gtid(input, header),
            0x23 => parse_previous_gtids(input, header),
            _ => unreachable!(),
        }
    }

    pub fn from_bytes<'a>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Event>> {
        let (i, _) = check_start(input)?;
        many1(Self::parse)(i)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub enum IntVarEventType {
    InvalidIntEvent,
    LastInsertIdEvent,
    InsertIdEvent,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct EmptyFlags {
    field_term_empty: bool,
    enclosed_empty: bool,
    line_term_empty: bool,
    line_start_empty: bool,
    escape_empty: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct OptFlags {
    dump_file: bool,
    opt_enclosed: bool,
    replace: bool,
    ignore: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub enum DupHandlingFlags {
    Error,
    Ignore,
    Replace,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub enum IncidentEventType {
    None,
    LostEvents,
}

fn parse_unknown<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    map(le_u32, move |checksum: u32| Event::Unknown {
        header: header.clone(),
        checksum,
    })(input)
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

fn parse_stop<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, checksum) = le_u32(input)?;
    Ok((i, Event::Stop { header, checksum }))
}

fn parse_rotate<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, position) = le_u64(input)?;
    let str_len = header.event_size - 19 - 8 - 4;
    let (i, next_binlog) = map(take(str_len), |s: &[u8]| string_var(s, str_len as usize))(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::Rotate {
            header,
            position,
            next_binlog,
            checksum,
        },
    ))
}

fn parse_intvar<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, e_type) = map(le_u8, |t: u8| match t {
        0x00 => IntVarEventType::InvalidIntEvent,
        0x01 => IntVarEventType::LastInsertIdEvent,
        0x02 => IntVarEventType::InsertIdEvent,
        _ => unreachable!(),
    })(input)?;
    let (i, (value, checksum)) = tuple((le_u64, le_u32))(i)?;
    Ok((
        i,
        Event::IntVar {
            header,
            e_type,
            value,
            checksum,
        },
    ))
}

fn extract_many_fields<'a>(
    input: &'a [u8],
    header: &Header,
    num_fields: u32,
    table_name_length: u8,
    schema_length: u8,
) -> IResult<&'a [u8], (Vec<u8>, Vec<String>, String, String, String)> {
    let (i, field_name_lengths) = map(take(num_fields), |s: &[u8]| s.to_vec())(input)?;
    let total_len: u64 = field_name_lengths.iter().sum::<u8>() as u64 + num_fields as u64;
    let (i, raw_field_names) = take(total_len)(i)?;
    let (_, field_names) =
        many_m_n(num_fields as usize, num_fields as usize, string_nul)(raw_field_names)?;
    let (i, table_name) = map(take(table_name_length + 1), |s: &[u8]| extract_string(s))(i)?;
    let (i, schema_name) = map(take(schema_length + 1), |s: &[u8]| extract_string(s))(i)?;
    let (i, file_name) = map(
        take(
            header.event_size as usize
                - 19
                - 25
                - num_fields as usize
                - total_len as usize
                - table_name_length as usize
                - schema_length as usize
                - 3
                - 4,
        ),
        |s: &[u8]| extract_string(s),
    )(i)?;
    Ok((
        i,
        (
            field_name_lengths,
            field_names,
            table_name,
            schema_name,
            file_name,
        ),
    ))
}

fn parse_load<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (
        i,
        (
            thread_id,
            execution_time,
            skip_lines,
            table_name_length,
            schema_length,
            num_fields,
            field_term,
            enclosed_by,
            line_term,
            line_start,
            escaped_by,
        ),
    ) = tuple((
        le_u32, le_u32, le_u32, le_u8, le_u8, le_u32, le_u8, le_u8, le_u8, le_u8, le_u8,
    ))(input)?;
    let (i, opt_flags) = map(le_u8, |flags: u8| OptFlags {
        dump_file: (flags >> 0) % 2 == 1,
        opt_enclosed: (flags >> 1) % 1 == 1,
        replace: (flags >> 2) % 2 == 1,
        ignore: (flags >> 3) % 2 == 1,
    })(i)?;
    let (i, empty_flags) = map(le_u8, |flags: u8| EmptyFlags {
        field_term_empty: (flags >> 0) % 2 == 1,
        enclosed_empty: (flags >> 1) % 2 == 1,
        line_term_empty: (flags >> 2) % 2 == 1,
        line_start_empty: (flags >> 3) % 2 == 1,
        escape_empty: (flags >> 4) % 2 == 1,
    })(i)?;
    let (i, (field_name_lengths, field_names, table_name, schema_name, file_name)) =
        extract_many_fields(i, &header, num_fields, table_name_length, schema_length)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::Load {
            header,
            thread_id,
            execution_time,
            skip_lines,
            table_name_length,
            schema_length,
            num_fields,
            field_term,
            enclosed_by,
            line_term,
            line_start,
            escaped_by,
            opt_flags,
            empty_flags,
            field_name_lengths,
            field_names,
            table_name,
            schema_name,
            file_name,
            checksum,
        },
    ))
}

fn parse_slave<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, checksum) = le_u32(input)?;
    Ok((i, Event::Slave { header, checksum }))
}

fn parse_file_data<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], (u32, String, u32)> {
    let (i, file_id) = le_u32(input)?;
    let (i, block_data) = map(take(header.event_size - 19 - 4 - 4), |s: &[u8]| {
        extract_string(s)
    })(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((i, (file_id, block_data, checksum)))
}

fn parse_create_file<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (file_id, block_data, checksum)) = parse_file_data(input, &header)?;
    Ok((
        i,
        Event::CreateFile {
            header,
            file_id,
            block_data,
            checksum,
        },
    ))
}

fn parse_append_file<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (file_id, block_data, checksum)) = parse_file_data(input, &header)?;
    Ok((
        i,
        Event::AppendFile {
            header,
            file_id,
            block_data,
            checksum,
        },
    ))
}

fn parse_exec_load<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    map(
        tuple((le_u16, le_u32)),
        |(file_id, checksum): (u16, u32)| Event::ExecLoad {
            header: header.clone(),
            file_id,
            checksum,
        },
    )(input)
}

fn parse_delete_file<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    map(
        tuple((le_u16, le_u32)),
        |(file_id, checksum): (u16, u32)| Event::DeleteFile {
            header: header.clone(),
            file_id,
            checksum,
        },
    )(input)
}

fn extract_from_prev<'a>(input: &'a [u8]) -> IResult<&'a [u8], (u8, String)> {
    let (i, len) = le_u8(input)?;
    map(take(len), move |s| (len, string_var(s, len as usize)))(i)
}

fn parse_new_load<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (thread_id, execution_time, skip_lines, table_name_length, schema_length, num_fields)) =
        tuple((le_u32, le_u32, le_u32, le_u8, le_u8, le_u32))(input)?;
    let (i, (field_term_length, field_term)) = extract_from_prev(i)?;
    let (i, (enclosed_by_length, enclosed_by)) = extract_from_prev(i)?;
    let (i, (line_term_length, line_term)) = extract_from_prev(i)?;
    let (i, (line_start_length, line_start)) = extract_from_prev(i)?;
    let (i, (escaped_by_length, escaped_by)) = extract_from_prev(i)?;
    let (i, opt_flags) = map(le_u8, |flags| OptFlags {
        dump_file: (flags >> 0) % 2 == 1,
        opt_enclosed: (flags >> 1) % 2 == 1,
        replace: (flags >> 2) % 2 == 1,
        ignore: (flags >> 3) % 2 == 1,
    })(i)?;
    let (i, (field_name_lengths, field_names, table_name, schema_name, file_name)) =
        extract_many_fields(i, &header, num_fields, table_name_length, schema_length)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::NewLoad {
            header,
            thread_id,
            execution_time,
            skip_lines,
            table_name_length,
            schema_length,
            num_fields,
            field_name_lengths,
            field_term,
            enclosed_by_length,
            enclosed_by,
            line_term_length,
            line_term,
            line_start_length,
            line_start,
            escaped_by_length,
            escaped_by,
            opt_flags,
            field_term_length,
            field_names,
            table_name,
            schema_name,
            file_name,
            checksum,
        },
    ))
}

fn parse_rand<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (seed1, seed2, checksum)) = tuple((le_u64, le_u64, le_u32))(input)?;
    Ok((
        i,
        Event::Rand {
            header,
            seed1,
            seed2,
            checksum,
        },
    ))
}

fn parse_user_var<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, name_length) = le_u32(input)?;
    let (i, name) = map(take(name_length), |s: &[u8]| {
        string_var(s, name_length as usize)
    })(i)?;
    let (i, is_null) = map(le_u8, |v| v == 1)(i)?;
    let (i, checksum) = le_u32(i)?;
    if is_null {
        Ok((
            i,
            Event::UserVar {
                header,
                name_length,
                name,
                is_null,
                d_type: None,
                charset: None,
                value_length: None,
                value: None,
                flags: None,
                checksum,
            },
        ))
    } else {
        let (i, d_type) = map(le_u8, |v| Some(v))(i)?;
        let (i, charset) = map(le_u32, |v| Some(v))(i)?;
        let (i, value_length) = le_u32(i)?;
        let (i, value) = map(take(value_length), |s: &[u8]| Some(s.to_vec()))(i)?;
        let (i, flags) = map(le_u8, |v| Some(v))(i)?;
        let (i, checksum) = le_u32(i)?;
        Ok((
            i,
            Event::UserVar {
                header,
                name,
                name_length,
                is_null,
                d_type,
                charset,
                value_length: Some(value_length),
                value,
                flags,
                checksum,
            },
        ))
    }
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

fn parse_xid<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (xid, checksum)) = tuple((le_u64, le_u32))(input)?;
    Ok((
        i,
        Event::XID {
            header,
            xid,
            checksum,
        },
    ))
}

fn parse_begin_load_query<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (file_id, block_data, checksum)) = parse_file_data(input, &header)?;
    Ok((
        i,
        Event::BeginLoadQuery {
            header,
            file_id,
            block_data,
            checksum,
        },
    ))
}

fn parse_execute_load_query<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (
        i,
        (
            thread_id,
            execution_time,
            schema_length,
            error_code,
            status_vars_length,
            file_id,
            start_pos,
            end_pos,
        ),
    ) = tuple((
        le_u32, le_u32, le_u8, le_u16, le_u16, le_u32, le_u32, le_u32,
    ))(input)?;
    let (i, dup_handling_flags) = map(le_u8, |flags| match flags {
        0 => DupHandlingFlags::Error,
        1 => DupHandlingFlags::Ignore,
        2 => DupHandlingFlags::Replace,
        _ => unreachable!(),
    })(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::ExecuteLoadQueryEvent {
            header,
            thread_id,
            execution_time,
            schema_length,
            error_code,
            status_vars_length,
            file_id,
            start_pos,
            end_pos,
            dup_handling_flags,
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
    // Reserved for future use; currently always 0
    let (i, flags) = le_u16(i)?;
    let (i, (schema_length, schema)) = string_fixed(i)?;
    let (i, term) = le_u8(i)?;
    assert_eq!(term, 0);

    let (i, (table_name_length, table_name)) = string_fixed(i)?;
    let (i, term) = le_u8(i)?;
    assert_eq!(term, 0);
    let (i, (_, column_count)) = int_lenenc(i)?;
    let (i, cols_type): (&'a [u8], Vec<ColTypes>) = map(take(column_count), |s: &[u8]| {
        s.iter().map(|&t| ColTypes::from_u8(t)).collect()
    })(i)?;
    let (i, (_, column_meta_count)) = int_lenenc(i)?;
    let (i, columns_type) = map(take(column_meta_count), |s: &[u8]| {
        let mut used = 0;
        let mut ret = vec![];
        for col in cols_type.iter() {
            let (_, (u, val)) = col.parse_def(&s[used..]).unwrap();
            used = used + u;
            ret.push(val);
        }
        ret
    })(i)?;
    let mask_len = (column_count + 7) / 8;
    let (i, null_bits) = map(take(mask_len), |s: &[u8]| s.to_vec())(i)?;
    let (i, checksum) = le_u32(i)?;
    if let Ok(mut mapping) = TABLE_MAP.lock() {
        mapping.insert(table_id, columns_type.clone());
    }
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
            columns_type,
            null_bits,
            checksum,
        },
    ))
}

fn parse_incident<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, d_type) = map(le_u16, |t| match t {
        0x0000 => IncidentEventType::None,
        0x0001 => IncidentEventType::LostEvents,
        _ => unreachable!(),
    })(input)?;
    let (i, message_length) = le_u8(i)?;
    let (i, message) = map(take(message_length), |s: &[u8]| {
        string_var(s, message_length as usize)
    })(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::Incident {
            header,
            d_type,
            message_length,
            message,
            checksum,
        },
    ))
}

fn parse_heartbeat<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, checksum) = le_u32(input)?;
    Ok((i, Event::Heartbeat { header, checksum }))
}

fn parse_row_query<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, length) = le_u8(input)?;
    let (i, query_text) = map(take(length), |s: &[u8]| string_var(s, length as usize))(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::RowQuery {
            header,
            length,
            query_text,
            checksum,
        },
    ))
}

fn parse_events_gtid<'a>(
    input: &'a [u8],
) -> IResult<&'a [u8], (bool, String, String, u8, i64, i64, u32)> {
    let (i, rbr_only) = map(le_u8, |t: u8| t == 0)(input)?;
    let (i, source_id) = map(take(16usize), |s: &[u8]| {
        format!(
            "{}-{}-{}-{}-{}",
            s[..4].iter().fold(String::new(), |mut acc, i| {
                acc.push_str(&i.to_string());
                acc
            }),
            s[4..6].iter().fold(String::new(), |mut acc, i| {
                acc.push_str(&i.to_string());
                acc
            }),
            s[6..8].iter().fold(String::new(), |mut acc, i| {
                acc.push_str(&i.to_string());
                acc
            }),
            s[8..10].iter().fold(String::new(), |mut acc, i| {
                acc.push_str(&i.to_string());
                acc
            }),
            s[10..].iter().fold(String::new(), |mut acc, i| {
                acc.push_str(&i.to_string());
                acc
            }),
        )
    })(i)?;
    let (i, transaction_id) = map(take(8usize), |s: &[u8]| {
        s.iter().fold(String::new(), |mut acc, i| {
            acc.push_str(&i.to_string());
            acc
        })
    })(i)?;
    let (i, ts_type) = le_u8(i)?;
    let (i, last_committed) = le_i64(i)?;
    let (i, sequence_number) = le_i64(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        (
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            checksum,
        ),
    ))
}

fn parse_anonymous_gtid<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    map(
        parse_events_gtid,
        |(
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            checksum,
        )| Event::AnonymousGtid {
            header: header.clone(),
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            checksum,
        },
    )(input)
}

fn parse_gtid<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    map(
        parse_events_gtid,
        |(
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            checksum,
        )| Event::Gtid {
            header: header.clone(),
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            checksum,
        },
    )(input)
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

fn parse_part_row_event<'a>(
    input: &'a [u8],
) -> IResult<&'a [u8], (u64, rows::Flags, u16, Vec<rows::ExtraData>, (usize, u64))> {
    let (i, table_id): (&'a [u8], u64) = map(take(6usize), |id_raw: &[u8]| {
        let mut filled = id_raw.to_vec();
        filled.extend(vec![0, 0]);
        pu64(&filled).unwrap().1
    })(input)?;
    let (i, flags) = map(le_u16, |flag: u16| rows::Flags {
        end_of_stmt: (flag >> 0) % 2 == 1,
        foreign_key_checks: (flag >> 1) % 2 == 0,
        unique_key_checks: (flag >> 2) % 2 == 0,
        has_columns: (flag >> 3) % 2 == 0,
    })(i)?;
    let (i, extra_data_len) = le_u16(i)?;
    assert!(extra_data_len >= 2);
    let (i, extra_data) = match extra_data_len {
        2 => (i, vec![]),
        _ => many1(rows::parse_extra_data)(i)?,
    };

    // parse body
    let (i, (encode_len, column_count)) = int_lenenc(i)?;
    Ok((
        i,
        (
            table_id,
            flags,
            extra_data_len,
            extra_data,
            (encode_len, column_count),
        ),
    ))
}

fn parse_row<'a>(
    input: &'a [u8],
    init_idx: usize,
    col_def: &Vec<ColTypes>,
) -> IResult<&'a [u8], Vec<ColValues>> {
    let mut index = if input.len() != 0 { init_idx } else { 0 };
    let mut ret = vec![];
    for col in col_def {
        let (_, (offset, col_val)) = col.parse(&input[index..])?;
        ret.push(col_val);
        index += offset;
    }
    Ok((&input[index..], ret))
}

fn parse_write_rows_v2<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (table_id, flags, extra_data_len, extra_data, (encode_len, column_count))) =
        parse_part_row_event(input)?;
    let bit_len = (column_count + 7) / 8;
    let (i, inserted_image_bits) = map(take(bit_len), |s: &[u8]| s.to_vec())(i)?;
    let (i, col_data) = take(
        header.event_size
            - 19
            - 6
            - 2
            - extra_data_len as u32
            - encode_len as u32
            - ((column_count as u32 + 7) / 8)
            - 4,
    )(i)?;
    let (_, rows) = many1(|s| {
        parse_row(
            s,
            bit_len as usize,
            TABLE_MAP.lock().unwrap().get(&table_id).unwrap(),
        )
    })(col_data)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::WriteRowsV2 {
            header,
            table_id,
            flags,
            extra_data_len,
            extra_data,
            column_count,
            inserted_image_bits,
            rows,
            checksum,
        },
    ))
}

fn parse_delete_rows_v2<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (table_id, flags, extra_data_len, extra_data, (encode_len, column_count))) =
        parse_part_row_event(input)?;

    let bit_len = (column_count + 7) / 8;
    let (i, deleted_image_bits) = map(take(bit_len), |s: &[u8]| s.to_vec())(i)?;
    let (i, col_data) = take(
        header.event_size
            - 19
            - 6
            - 2
            - extra_data_len as u32
            - encode_len as u32
            - ((column_count as u32 + 7) / 8)
            - 4,
    )(i)?;
    let (_, rows) = many1(|s| {
        parse_row(
            s,
            bit_len as usize,
            TABLE_MAP.lock().unwrap().get(&table_id).unwrap(),
        )
    })(col_data)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::DeleteRowsV2 {
            header,
            table_id,
            flags,
            extra_data_len,
            extra_data,
            column_count,
            deleted_image_bits,
            rows,
            checksum,
        },
    ))
}

fn parse_update_rows_v2<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, (table_id, flags, extra_data_len, extra_data, (encode_len, column_count))) =
        parse_part_row_event(input)?;

    let bit_len = (column_count + 7) / 8;
    let (i, before_image_bits) = map(take(bit_len), |s: &[u8]| s.to_vec())(i)?;
    let (i, after_image_bits) = map(take(bit_len), |s: &[u8]| s.to_vec())(i)?;
    // TODO I still don't know is it right or not :(
    let (i, col_data) = take(
        header.event_size as u64
            - 19
            - 6
            - 2
            - extra_data_len as u64
            - encode_len as u64
            - bit_len * 2
            - 4,
    )(i)?;
    let (_, rows) = many1(|s| {
        parse_row(
            s,
            bit_len as usize,
            TABLE_MAP.lock().unwrap().get(&table_id).unwrap(),
        )
    })(col_data)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::UpdateRowsV2 {
            header,
            table_id,
            flags,
            extra_data_len,
            extra_data,
            column_count,
            before_image_bits,
            after_image_bits,
            rows,
            checksum,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_anonymous_gtids() {
        let input = include_bytes!("../../tests/bin_files/anonymous_gtids1.bin");
        let (i, header) = parse_header(input).unwrap();
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
        let input = include_bytes!("../../tests/bin_files/format_desc1.bin");
        let (i, header) = parse_header(input).unwrap();
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
    fn test_xid() {
        let input = include_bytes!("../../tests/bin_files/xid1.bin");
        let (i, header) = parse_header(input).unwrap();
        let (i, e) = parse_xid(i, header).unwrap();
        match e {
            Event::XID { xid, checksum, .. } => {
                assert_eq!(i.len(), 0);
                assert_eq!(xid, 11);
                assert_eq!(checksum, 0x86eb78bc);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_previous_gtids() {
        use super::parse_header;

        let input = include_bytes!("../../tests/bin_files/previous_gtids1.bin");
        let (i, header) = parse_header(input).unwrap();
        let (i, _) = parse_previous_gtids(i, header).unwrap();
        assert_eq!(i.len(), 0);
    }

    #[test]
    fn test_table_map() {
        use super::parse_header;

        let input = include_bytes!("../../tests/bin_files/table_map1.bin");
        let (i, header) = parse_header(input).unwrap();
        let (i, event) = parse_table_map(i, header).unwrap();
        match event {
            Event::TableMap {
                table_id,
                schema,
                checksum,
                ..
            } => {
                assert_eq!(i.len(), 0);
                // TODO do more checks here
                assert_eq!(table_id, 109);
                assert_eq!(schema, "test".to_string());
                assert_eq!(checksum, 0x4435a8c2);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_query() {
        use super::parse_header;

        let input = include_bytes!("../../tests/bin_files/query1.bin");
        let (i, header) = parse_header(input).unwrap();
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

    #[test]
    fn test_write_row_v2() {
        let input = include_bytes!("../../tests/bin_files/write_rows_v21.bin");
        let (i, header) = parse_header(input).unwrap();
        let (i, _) = parse_table_map(i, header).unwrap();
        let (i, header) = parse_header(i).unwrap();
        let (i, e) = parse_write_rows_v2(&i, header).unwrap();
        match e {
            Event::WriteRowsV2 {
                table_id,
                flags,
                checksum,
                ..
            } => {
                assert_eq!(i.len(), 0);
                assert_eq!(table_id, 115);
                assert_eq!(checksum, 0x73cbfb1e);
                assert_eq!(
                    flags,
                    rows::Flags {
                        end_of_stmt: true,
                        foreign_key_checks: true,
                        unique_key_checks: true,
                        has_columns: true
                    }
                )
            }
            _ => unreachable!(),
        }
    }
}
