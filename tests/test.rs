use boxercrab::events::Event;
use boxercrab::events::Event::*;
use boxercrab::mysql::ColValues::*;

#[test]
fn test_stop() {
    let input = include_bytes!("events/03_stop/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(2).unwrap() {
        Stop { .. } => {}
        _ => panic!("should be stop event"),
    }
}

#[test]
fn test_rotate() {
    let input = include_bytes!("events/04_rotate/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(2).unwrap() {
        Rotate {
            next_binlog,
            position,
            ..
        } => {
            assert_eq!(next_binlog, "mysql_bin.000002");
            assert_eq!(*position, 4);
        }
        _ => panic!("should be rotate"),
    }
}

#[test]
fn test_intvar() {
    use boxercrab::events::IntVarEventType;
    let input = include_bytes!("events/05_intvar/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(8).unwrap() {
        IntVar { e_type, value, .. } => {
            assert_eq!(e_type, &IntVarEventType::LastInsertIdEvent);
            assert_eq!(*value, 0);
        }
        _ => panic!("should be intvar"),
    }
}

#[test]
fn test_rand() {
    let input = include_bytes!("events/13_rand/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(8).unwrap() {
        Rand { seed1, seed2, .. } => {
            assert_eq!(*seed1, 694882935);
            assert_eq!(*seed2, 292094996);
        }
        _ => panic!("should be rand"),
    }
}

#[test]
fn test_user_var() {
    use boxercrab::events::UserVarType;

    let input = include_bytes!("events/14_user_var/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    // TODO need to test other types & null var
    match output.get(9).unwrap() {
        UserVar {
            name,
            d_type,
            charset,
            value,
            ..
        } => {
            assert_eq!(name, "val_s");
            assert_eq!(*d_type, Some(UserVarType::STRING));
            assert_eq!(*charset, Some(33));
            assert_eq!(
                *value,
                Some(vec![116, 101, 115, 116, 32, 98, 108, 111, 103])
            )
        }
        _ => panic!("should be user var"),
    }
    match output.get(10).unwrap() {
        UserVar {
            name,
            d_type,
            charset,
            value,
            ..
        } => {
            assert_eq!(name, "val_i");
            assert_eq!(*d_type, Some(UserVarType::INT));
            assert_eq!(*charset, Some(33));
            assert_eq!(*value, Some(vec![100, 0, 0, 0, 0, 0, 0, 0]))
        }
        _ => panic!("should be user var"),
    }
    match output.get(11).unwrap() {
        UserVar {
            name,
            d_type,
            charset,
            value,
            ..
        } => {
            assert_eq!(name, "val_d");
            assert_eq!(*d_type, Some(UserVarType::DECIMAL));
            assert_eq!(*charset, Some(33));
            assert_eq!(*value, Some(vec![03, 02, 129, 0]))
        }
        _ => panic!("should be user var"),
    }
}

#[test]
fn test_format_desc() {
    let input = include_bytes!("events/15_format_desc/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(0).unwrap() {
        FormatDesc {
            binlog_version,
            mysql_server_version,
            create_timestamp,
            ..
        } => {
            assert_eq!(*binlog_version, 4);
            assert_eq!(mysql_server_version, "5.7.30-log");
            assert_eq!(*create_timestamp, 1596175634)
        }
        _ => panic!("should be format desc"),
    }
}

#[test]
fn test_xid() {
    let input = include_bytes!("events/16_xid/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(10).unwrap() {
        XID { xid, .. } => {
            assert_eq!(*xid, 41);
        }
        _ => panic!("should be xid"),
    }
}

#[test]
fn test_table_map() {
    use boxercrab::mysql::ColTypes::*;

    // TODO need to test more column types
    let input = include_bytes!("events/19_table_map/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(8).unwrap() {
        TableMap {
            table_id,
            table_name,
            flags,
            columns_type,
            null_bits,
            ..
        } => {
            assert_eq!(*table_id, 110);
            assert_eq!(table_name, "boxercrab");
            assert_eq!(*flags, 1);
            assert_eq!(*columns_type, vec![Long, VarChar(160)]);
            assert_eq!(*null_bits, vec![0]);
        }
        _ => panic!("should be table_map"),
    }
}

#[test]
fn test_row_query() {
    let input = include_bytes!("events/29_row_query/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(8).unwrap() {
        RowQuery { query_text, .. } => assert_eq!(
            query_text,
            "INSERT INTO `boxercrab` (`title`) VALUES ('hahhhhhhhhh')"
        ),
        _ => panic!("should be row_query"),
    }
}

#[test]
fn test_begin_load_query_and_exec_load_query() {
    let input = include_bytes!("events/17_18_load/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(4).unwrap() {
        BeginLoadQuery {
            file_id,
            block_data,
            ..
        } => {
            assert_eq!(*file_id, 1);
            assert_eq!(block_data, "1,\"abc\"\n");
        }
        _ => panic!("should be begin load query"),
    };
    match output.get(5).unwrap() {
        ExecuteLoadQueryEvent {
            thread_id,
            file_id,
            start_pos,
            end_pos,
            schema,
            query,
            ..
        } => {
            assert_eq!(*thread_id, 23);
            assert_eq!(*file_id, 1);
            assert_eq!(*start_pos, 9);
            assert_eq!(*end_pos, 37);
            assert_eq!(schema, "default");
            assert_eq!(query, "LOAD DATA INFILE '/tmp/data.txt' INTO TABLE `boxercrab` FIELDS TERMINATED BY ',' OPTIONALLY  ENCLOSED BY '\"' ESCAPED BY '\\\\' LINES TERMINATED BY '\\n' (`i`, `c`)");
        }
        _ => panic!("should be exec load query"),
    }
}

#[test]
fn test_write_rows_v2() {
    let input = include_bytes!("events/30_write_rows_v2/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(10).unwrap() {
        WriteRowsV2 {
            table_id,
            column_count,
            rows,
            ..
        } => {
            assert_eq!(*table_id, 111);
            assert_eq!(*column_count, 2);
            assert_eq!(
                *rows,
                vec![vec![
                    Long(vec![1, 0, 0, 0]),
                    VarChar(vec![97, 98, 99, 100, 101])
                ]]
            )
        }
        _ => panic!("should write_rows_v2"),
    }
}

#[test]
fn test_update_rows_v2() {
    let input = include_bytes!("events/31_update_rows_v2/log.bin");
    let (_, output) = Event::from_bytes(input).unwrap();
    let update_row = output.get(5).unwrap();
    let abc = vec![97, 98, 99];
    let xd = vec![120, 100];
    let values = vec![
        vec![
            Long(vec![1, 0, 0, 0]),
            VarChar(abc.clone()),
            VarChar(abc.clone()),
            Blob(abc.clone()),
            Blob(abc.clone()),
            Blob(abc.clone()),
            Float(1.0),
            Double(2.0),
            NewDecimal(vec![128, 0, 3, 0, 0]),
        ],
        vec![
            Long(vec![1, 0, 0, 0]),
            VarChar(xd.clone()),
            VarChar(xd.clone()),
            Blob(xd.clone()),
            Blob(xd.clone()),
            Blob(xd.clone()),
            Float(4.0),
            Double(4.0),
            NewDecimal(vec![128, 0, 4, 0, 0]),
        ],
    ];
    match update_row {
        UpdateRowsV2 { rows, .. } => assert_eq!(rows, &values),
        _ => panic!("should be update_row_v2"),
    }
}

#[test]
fn test_delete_rows_v2() {
    let input = include_bytes!("events/32_delete_rows_v2/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(16).unwrap() {
        DeleteRowsV2 {
            table_id,
            column_count,
            rows,
            ..
        } => {
            assert_eq!(*table_id, 112);
            assert_eq!(*column_count, 2);
            assert_eq!(
                *rows,
                vec![vec![
                    Long(vec![1, 0, 0, 0]),
                    VarChar(vec![97, 98, 99, 100, 101])
                ]]
            )
        }
        _ => panic!("should be delete rows v2"),
    }
}

#[test]
fn test_gtid() {
    let input = include_bytes!("events/33_35_gtid_prev_gtid/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(2).unwrap() {
        Gtid {
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            ..
        } => {
            assert_eq!(*rbr_only, false);
            assert_eq!(source_id, "12884158204-210242-17234-183144-2661721902");
            assert_eq!(transaction_id, "10000000");
            assert_eq!(*ts_type, 2);
            assert_eq!(*last_committed, 0);
            assert_eq!(*sequence_number, 1);
        }
        _ => panic!("should be gtid"),
    }
}

#[test]
fn test_anonymous_gtid() {
    let input = include_bytes!("events/34_anonymous_gtid/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(2).unwrap() {
        AnonymousGtid {
            rbr_only,
            source_id,
            transaction_id,
            ts_type,
            last_committed,
            sequence_number,
            ..
        } => {
            assert_eq!(*rbr_only, false);
            assert_eq!(source_id, "0000-00-00-00-000000");
            assert_eq!(transaction_id, "00000000");
            assert_eq!(*ts_type, 2);
            assert_eq!(*last_committed, 0);
            assert_eq!(*sequence_number, 1);
        }
        _ => panic!("should be anonymous gtid"),
    }
}

#[test]
fn test_previous_gtid() {
    let input = include_bytes!("events/33_35_gtid_prev_gtid/log.bin");
    let (remain, output) = Event::from_bytes(input).unwrap();
    assert_eq!(remain.len(), 0);
    match output.get(1).unwrap() {
        PreviousGtids { gtid_sets, .. } => {
            assert_eq!(*gtid_sets, vec![0, 0, 0, 0]);
        }
        _ => panic!("should be previous gtid"),
    }
}
