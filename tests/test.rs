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
        _ => panic!("should be rotate event"),
    }
}

// #[test]
// fn test_intvar() {
//     let input = include_bytes!("events/05_intvar/log.bin");
//     let (remain, output) = Event::from_bytes(input).unwrap();
//     assert_eq!(remain.len(), 0);
// }

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
        ],
        vec![
            Long(vec![1, 0, 0, 0]),
            VarChar(xd.clone()),
            VarChar(xd.clone()),
            Blob(xd.clone()),
            Blob(xd.clone()),
            Blob(xd.clone()),
        ],
    ];
    match update_row {
        UpdateRowsV2 { rows, .. } => assert_eq!(rows, &values),
        _ => panic!("should not reach"),
    }
}
