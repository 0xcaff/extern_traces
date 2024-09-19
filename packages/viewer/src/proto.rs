use std::io;
use std::io::{ErrorKind, Read};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub enum TraceEvent {
    Start(InitialMessage),
    Span(SpanEvent),
}

#[derive(Debug, Clone)]
pub struct InitialMessage {
    pub tsc_frequency: u64,
    pub anchor_seconds: i64,
    pub anchor_nanoseconds: i64,
    pub anchor_timestamp: u64,
}

impl InitialMessage {
    pub fn anchor(&self) -> SystemTime {
        let duration_since_epoch =
            Duration::new(self.anchor_seconds as u64, self.anchor_nanoseconds as u32);

        UNIX_EPOCH + duration_since_epoch
    }
}

#[derive(Debug, Clone)]
pub enum SpanEvent {
    Start(SpanStart),
    End(SpanEnd),
}

#[derive(Debug, Clone)]
pub struct SpanStart {
    pub thread_id: u64,
    pub time: u64,
    pub label_id: u64,
}

#[derive(Debug, Clone)]
pub struct SpanEnd {
    pub thread_id: u64,
    pub time: u64,
}

impl SpanEvent {
    pub fn read(mut stream: impl Read) -> io::Result<SpanEvent> {
        let mut message_tag = [0u8; 8];
        stream.read_exact(&mut message_tag)?;
        let message_tag = u64::from_le_bytes(message_tag);

        match message_tag {
            0 => {
                // SpanStart
                let mut data = [0u8; 24];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());
                let label_id = u64::from_le_bytes(data[16..24].try_into().unwrap());
                Ok(SpanEvent::Start(SpanStart {
                    thread_id,
                    time,
                    label_id,
                }))
            }
            1 => {
                // SpanEnd
                let mut data = [0u8; 16];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());
                Ok(SpanEvent::End(SpanEnd { thread_id, time }))
            }
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

impl InitialMessage {
    pub fn read(mut stream: impl Read) -> io::Result<InitialMessage> {
        let mut initial_message_buf = [0u8; 8 * 4];
        stream.read_exact(&mut initial_message_buf)?;
        let tsc_frequency = u64::from_le_bytes(initial_message_buf[0..8].try_into().unwrap());
        let anchor_seconds = i64::from_le_bytes(initial_message_buf[8..16].try_into().unwrap());
        let anchor_nanoseconds =
            i64::from_le_bytes(initial_message_buf[16..24].try_into().unwrap());
        let anchor_timestamp = u64::from_le_bytes(initial_message_buf[24..32].try_into().unwrap());

        let initial_message = InitialMessage {
            tsc_frequency,
            anchor_seconds,
            anchor_nanoseconds,
            anchor_timestamp,
        };

        Ok(initial_message)
    }
}
