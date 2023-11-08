use parse_tool::InputBuf;

use crate::codec::{Decode, DecodeError, Int1, Int2, Int4, Int8};

bitflags::bitflags! {
    /// https://dev.mysql.com/doc/dev/mysql-server/latest/group__group__cs__binglog__event__header__flags.html
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[cfg_attr(feature="serde", serde::Serialize, serde::DeSerialize)]
    pub struct EventHeaderFlag: u16 {
        /// If the query depends on the thread (for example: TEMPORARY TABLE)
        const LOG_EVENT_THREAD_SPECIFIC_F=   0x4;
        /// Suppress the generation of 'USE' statements before the actual statement
        const LOG_EVENT_SUPPRESS_USE_F   =0x8;
        /// Artificial events are created arbitrarily and not written to binary log
        const LOG_EVENT_ARTIFICIAL_F =    0x20;
        /// Events with this flag set are created by slave IO thread and written to relay log
        const LOG_EVENT_RELAY_LOG_F =    0x40;
        /// For an event, 'e', carrying a type code, that a slave, 's', does not recognize, 's' will check 'e' for LOG_EVENT_IGNORABLE_F, and if the flag is set, then 'e' is ignored
        const LOG_EVENT_IGNORABLE_F =    0x80;
        /// Events with this flag are not filtered
        const LOG_EVENT_NO_FILTER_F =    0x100;
        /// MTS: group of events can be marked to force its execution in isolation from any other Workers
        const LOG_EVENT_MTS_ISOLATE_F =    0x200;
    }
}

impl<I: InputBuf> Decode<I> for EventHeaderFlag {
    fn decode(input: &mut I) -> Result<Self, crate::codec::DecodeError> {
        let flags = Int2::decode(input)?;
        Self::from_bits(flags.int()).ok_or(DecodeError::InvalidData)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct EventHeader {
    pub timestamp: Int4,
    pub event_type: Int1,
    pub server_id: Int4,
    pub event_size: Int4,
    pub log_pos: Int4,
    pub flags: EventHeaderFlag,
}

impl<I: InputBuf> Decode<I> for EventHeader {
    fn decode(input: &mut I) -> Result<Self, DecodeError> {
        let timestamp = Int4::decode(input)?;
        let event_type = Int1::decode(input)?;
        let server_id = Int4::decode(input)?;
        let event_size = Int4::decode(input)?;
        let log_pos = Int4::decode(input)?;
        let flags = EventHeaderFlag::decode(input)?;

        Ok(Self {
            timestamp,
            event_type,
            server_id,
            event_size,
            log_pos,
            flags,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct EventRaw {
    pub header: EventHeader,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct Event<P> {
    pub header: EventHeader,
    pub payload: P,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct QueryEvent {
    /// thread id
    pub slave_proxy_id: Int4,
    pub exec_time: Int4,
    pub schema_len: Int1,
    pub error_code: Int2,
    pub status_vars_length: Int2,
}

pub enum QueryStatusVar {
    QFlags2Code(),
}

bitflags::bitflags! {
    /// bit mask of flags that are usually set with the SET command
    ///
    /// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_replication_binlog_event.html#sect_protocol_replication_event_query_00)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[cfg_attr(feature="serde", serde::Serialize, serde::DeSerialize)]
    pub struct QFlag2Code : u32 {
        const OPTION_AUTO_IS_NULL =  0x00004000;
        const OPTION_NOT_AUTOCOMMIT =  0x00080000;
        const OPTION_NO_FOREIGN_KEY_CHECKS =  0x04000000;
        const OPTION_RELAXED_UNIQUE_CHECKS =  0x08000000;
    }

    /// bit mask of flags that are usually set with SET sql_mode
    ///
    /// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_replication_binlog_event.html#sect_protocol_replication_event_query_00)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[cfg_attr(feature="serde", serde::Serialize, serde::DeSerialize)]
    pub struct QSqlModeCode : u64 {
        const MODE_REAL_AS_FLOAT = 0x00000001;
        const MODE_PIPES_AS_CONCAT = 0x00000002;
        const MODE_ANSI_QUOTES = 0x00000004;
        const MODE_IGNORE_SPACE = 0x00000008;
        const MODE_NOT_USED = 0x00000010;
        const MODE_ONLY_FULL_GROUP_BY = 0x00000020;
        const MODE_NO_UNSIGNED_SUBTRACTION = 0x00000040;
        const MODE_NO_DIR_IN_CREATE = 0x00000080;
        const MODE_POSTGRESQL = 0x00000100;
        const MODE_ORACLE = 0x00000200;
        const MODE_MSSQL = 0x00000400;
        const MODE_DB2 = 0x00000800;
        const MODE_MAXDB = 0x00001000;
        const MODE_NO_KEY_OPTIONS = 0x00002000;
        const MODE_NO_TABLE_OPTIONS = 0x00004000;
        const MODE_NO_FIELD_OPTIONS = 0x00008000;
        const MODE_MYSQL323 = 0x00010000;
        const MODE_MYSQL40 = 0x00020000;
        const MODE_ANSI = 0x00040000;
        const MODE_NO_AUTO_VALUE_ON_ZERO = 0x00080000;
        const MODE_NO_BACKSLASH_ESCAPES = 0x00100000;
        const MODE_STRICT_TRANS_TABLES = 0x00200000;
        const MODE_STRICT_ALL_TABLES = 0x00400000;
        const MODE_NO_ZERO_IN_DATE = 0x00800000;
        const MODE_NO_ZERO_DATE = 0x01000000;
        const MODE_INVALID_DATES = 0x02000000;
        const MODE_ERROR_FOR_DIVISION_BY_ZERO = 0x04000000;
        const MODE_TRADITIONAL = 0x08000000;
        const MODE_NO_AUTO_CREATE_USER = 0x10000000;
        const MODE_HIGH_NOT_PRECEDENCE = 0x20000000;
        const MODE_NO_ENGINE_SUBSTITUTION = 0x40000000;
        const MODE_PAD_CHAR_TO_FULL_LENGTH = 0x80000000;
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct QAutoIncrement {}

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_replication_binlog_event.html#sect_protocol_replication_event_stop)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]

pub struct StopEvent;

/// [doc](https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_replication_binlog_event.html#sect_protocol_replication_event_rotate)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct RotateEvent {
    pub pos: Int8,
    pub log: String,
}

/// [source](https://dev.mysql.com/doc/dev/mysql-server/latest/classbinary__log_1_1Intvar__event.html#details)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::DeSerialize)]
pub struct IntVarEvent {
    pub ty: u8,
    pub value: u64,
}

#[repr(u8)]
pub enum IntVarEventType {
    InvalidIntEvent = 0,
    LastInsertIdEvent = 1,
    InsertIdEvent = 2,
}
