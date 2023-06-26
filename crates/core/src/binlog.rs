use parse_tool::InputBuf;

use crate::codec::{Decode, DecodeError, Int1, Int2, Int4};

bitflags::bitflags! {
    /// https://dev.mysql.com/doc/dev/mysql-server/latest/group__group__cs__binglog__event__header__flags.html
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
pub struct Event {
    pub header: EventHeader,
    pub payload: Vec<u8>,
}

pub struct QueryEvent {
    /// thread id
    pub slave_proxy_id: Int4,
    pub exec_time: Int4,
    pub schema_len: Int1,
    pub error_code: Int2,
    pub status_vars_length: Int2,
}
