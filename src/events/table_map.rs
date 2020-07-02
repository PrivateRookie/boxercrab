use super::{Event, Header, Parse};
use crate::utils::{extract_string, parse_lenenc_int};
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_u16, le_u32, le_u64, le_u8},
    IResult,
};

// source: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/rows_event.h#L59-L373
// layout: https://github.com/mysql/mysql-server/blob/a394a7e17744a70509be5d3f1fd73f8779a31424/libbinlogevents/include/rows_event.h#L387-L401
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TableMap {
    pub header: Header,
    // table_id take 6 bytes in buffer
    pub table_id: u64,
    pub flags: u16,
    pub schema_length: u8,
    pub schema: String,
    // [00] term sign in layout
    pub table_name_length: u8,
    pub table_name: String,
    // [00] term sign in layout
    // len encoded integer
    pub column_count: u64,
    pub column_type_def: Vec<u8>,
    // len encoded string
    pub column_meta_def: Vec<u8>,
    pub null_bit_mask: Vec<u8>,
    pub checksum: u32,
}

impl Parse<TableMap> for TableMap {
    fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], TableMap> {
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
            TableMap {
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
}

pub fn parse<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let f = move |i| TableMap::parse(i, header.clone());
    map(f, |e| Event::TableMap(e))(input)
}

fn pu64(input: &[u8]) -> IResult<&[u8], u64> {
    le_u64(input)
}

#[test]
fn test_table_map() {
    use super::parse_header;

    let input: Vec<u8> = vec![
        170, 157, 253, 94, 19, 123, 0, 0, 0, 60, 0, 0, 0, 246, 2, 0, 0, 0, 0, 109, 0, 0, 0, 0, 0,
        1, 0, 4, 116, 101, 115, 116, 0, 10, 114, 117, 110, 111, 111, 98, 95, 116, 98, 108, 0, 4, 3,
        15, 15, 10, 4, 44, 1, 120, 0, 8, 194, 168, 53, 68,
    ];
    let (i, header) = parse_header(&input).unwrap();
    let (i, event) = TableMap::parse(i, header).unwrap();
    assert_eq!(i.len(), 0);
    // TODO do more checks here
    assert_eq!(event.table_id, 109);
    assert_eq!(event.schema, "test".to_string());
}
