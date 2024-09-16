use anyhow::bail;
use std::io::Read;

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

#[derive(Debug, Clone)]
pub enum SpanEvent {
    Start(SpanStart),
    End(SpanEnd),
}

impl SpanEvent {
    pub fn read(mut stream: impl Read) -> anyhow::Result<SpanEvent> {
        let mut message_tag = [0u8; 8];
        stream.read_exact(&mut message_tag)?;
        let message_tag = u64::from_le_bytes(message_tag);

        match message_tag {
            0 => {
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
                let mut data = [0u8; 16];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());

                Ok(SpanEvent::End(SpanEnd { thread_id, time }))
            }
            _ => bail!("unknown message tag: {}", message_tag),
        }
    }
}
