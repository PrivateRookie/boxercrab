#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColumnTypes {
    Decimal,
    Tiny,
    Short,
    Long,
    Float,
    Double,
    Null,
    Timestamp,
    LongLong,
    Int24,
    Date,
    Time,
    DateTime,
    Year,
    NewDate, // internal used
    VarChar,
    Bit,
    NewDecimal,
    Enum,       // internal used
    Set,        // internal used
    TinyBlob,   // internal used
    MediumBlob, // internal used
    LongBlob,   // internal used
    Blob,
    VarString,
    String,
    Geometry,
}

impl ColumnTypes {
    /// return (identifer, bytes used) of column type
    pub fn meta(&self) -> (u8, u8) {
        match *self {
            ColumnTypes::Decimal => (0, 0),
            ColumnTypes::Tiny => (1, 0),
            ColumnTypes::Short => (2, 0),
            ColumnTypes::Long => (3, 0),
            ColumnTypes::Float => (4, 1),
            ColumnTypes::Double => (5, 1),
            ColumnTypes::Null => (6, 0),
            ColumnTypes::Timestamp => (7, 0),
            ColumnTypes::LongLong => (8, 0),
            ColumnTypes::Int24 => (9, 0),
            ColumnTypes::Date => (10, 0),
            ColumnTypes::Time => (11, 0),
            ColumnTypes::DateTime => (12, 0),
            ColumnTypes::Year => (13, 0),
            ColumnTypes::NewDate => (14, 0),
            ColumnTypes::VarChar => (15, 2),
            ColumnTypes::Bit => (16, 2),
            ColumnTypes::NewDecimal => (246, 2),
            ColumnTypes::Enum => (247, 0),
            ColumnTypes::Set => (248, 0),
            ColumnTypes::TinyBlob => (249, 0),
            ColumnTypes::MediumBlob => (250, 0),
            ColumnTypes::LongBlob => (251, 0),
            ColumnTypes::Blob => (252, 1),
            ColumnTypes::VarString => (253, 2),
            ColumnTypes::String => (254, 2),
            ColumnTypes::Geometry => (255, 1),
        }
    }

    pub fn from_u8(t: u8) -> Self {
        match t {
            0 => ColumnTypes::Decimal,
            1 => ColumnTypes::Tiny,
            2 => ColumnTypes::Short,
            3 => ColumnTypes::Long,
            4 => ColumnTypes::Float,
            5 => ColumnTypes::Double,
            6 => ColumnTypes::Null,
            7 => ColumnTypes::Timestamp,
            8 => ColumnTypes::LongLong,
            9 => ColumnTypes::Int24,
            10 => ColumnTypes::Date,
            11 => ColumnTypes::Time,
            12 => ColumnTypes::DateTime,
            13 => ColumnTypes::Year,
            14 => ColumnTypes::NewDate,
            15 => ColumnTypes::VarChar,
            16 => ColumnTypes::Bit,
            246 => ColumnTypes::NewDecimal,
            247 => ColumnTypes::Enum,
            248 => ColumnTypes::Set,
            249 => ColumnTypes::TinyBlob,
            250 => ColumnTypes::MediumBlob,
            251 => ColumnTypes::LongBlob,
            252 => ColumnTypes::Blob,
            253 => ColumnTypes::VarString,
            254 => ColumnTypes::String,
            255 => ColumnTypes::Geometry,
            _ => {
                log::error!("unknown column type: {}", t);
                unreachable!()
            }
        }
    }
}
