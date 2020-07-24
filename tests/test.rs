#[test]
fn test_update_row_v2() {
    use boxercrab::events::Event;
    use boxercrab::mysql::ColValues::*;

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
        Event::UpdateRowsV2 { rows, .. } => assert_eq!(rows, &values),
        _ => panic!("should not reach"),
    }
}
