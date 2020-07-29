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
fn test_update_row_v2() {
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