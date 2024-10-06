use std::collections::BTreeMap;
use std::io;
use std::io::{ErrorKind, Read};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[repr(u8)]
pub enum TraceCommand {
    CaptureFrame = 0x0,
}

pub enum TraceEvent {
    Start(InitialMessage),
    Span(SpanEvent),
    CountersUpdate(CountersUpdate),
}

#[derive(Debug, Clone)]
pub struct InitialMessage {
    pub tsc_frequency: u64,
    pub anchor_seconds: i64,
    pub anchor_nanoseconds: i64,
    pub anchor_timestamp: u64,
    pub modules: BTreeMap<u16, ModuleInfo>,
    pub libraries: BTreeMap<u16, LibraryInfo>,
    pub symbols: Vec<SymbolInfo>,
}

impl InitialMessage {
    pub fn anchor(&self) -> SystemTime {
        let duration_since_epoch =
            Duration::new(self.anchor_seconds as u64, self.anchor_nanoseconds as u32);

        UNIX_EPOCH + duration_since_epoch
    }

    pub fn read(mut stream: impl Read) -> io::Result<InitialMessage> {
        let mut initial_message_buf = [0u8; 8 * 4];
        stream.read_exact(&mut initial_message_buf)?;
        let tsc_frequency = u64::from_le_bytes(initial_message_buf[0..8].try_into().unwrap());
        let anchor_seconds = i64::from_le_bytes(initial_message_buf[8..16].try_into().unwrap());
        let anchor_nanoseconds =
            i64::from_le_bytes(initial_message_buf[16..24].try_into().unwrap());
        let anchor_timestamp = u64::from_le_bytes(initial_message_buf[24..32].try_into().unwrap());

        let modules = {
            let module_count = read_u32(&mut stream)?;
            let mut modules = BTreeMap::new();
            for _ in 0..module_count {
                let module_info = ModuleInfo::parse(&mut stream)?;
                modules.insert(module_info.id, module_info);
            }
            modules
        };

        let libraries = {
            let library_count = read_u32(&mut stream)?;
            let mut libraries = BTreeMap::new();

            for _ in 0..library_count {
                let library_info = LibraryInfo::parse(&mut stream)?;
                libraries.insert(library_info.id, library_info);
            }

            libraries
        };

        let symbols = {
            let symbol_count = read_u32(&mut stream)?;
            let mut symbols = Vec::with_capacity(symbol_count as usize);
            for _ in 0..symbol_count {
                symbols.push(SymbolInfo::parse(&mut stream)?);
            }

            symbols
        };

        let initial_message = InitialMessage {
            tsc_frequency,
            anchor_seconds,
            anchor_nanoseconds,
            anchor_timestamp,
            modules,
            libraries,
            symbols,
        };

        Ok(initial_message)
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
    pub extra_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct SpanEnd {
    pub thread_id: u64,
    pub time: u64,
}

#[derive(Debug, Clone)]
pub struct CountersUpdate {
    pub thread_id: u64,
    pub dropped_packets_delta: u64,
    pub last_time: u64,
    pub time: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: u16,
    pub version_major: u8,
    pub version_minor: u8,
    pub name: String,
}

impl ModuleInfo {
    pub fn parse(stream: &mut impl Read) -> io::Result<Self> {
        let mut module_data = [0u8; 4];
        stream.read_exact(&mut module_data)?;
        let id = u16::from_le_bytes(module_data[0..2].try_into().unwrap());
        let version_major = module_data[2];
        let version_minor = module_data[3];
        let name = read_string(stream)?;

        Ok(ModuleInfo {
            id,
            version_major,
            version_minor,
            name,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub id: u16,
    pub version: u16,
    pub name: String,
}

impl LibraryInfo {
    pub fn parse(stream: &mut impl Read) -> io::Result<Self> {
        let mut library_data = [0u8; 4];
        stream.read_exact(&mut library_data)?;
        let id = u16::from_le_bytes(library_data[0..2].try_into().unwrap());
        let version = u16::from_le_bytes(library_data[2..4].try_into().unwrap());
        let name = read_string(stream)?;

        Ok(LibraryInfo { id, version, name })
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub library_id: u8,
    pub module_id: u8,
}

impl SymbolInfo {
    pub fn parse(stream: &mut impl Read) -> io::Result<Self> {
        let name = read_string(stream)?;
        let mut symbol_data = [0u8; 2];
        stream.read_exact(&mut symbol_data)?;
        let library_id = symbol_data[0];
        let module_id = symbol_data[1];

        Ok(SymbolInfo {
            name,
            library_id,
            module_id,
        })
    }
}

impl TraceEvent {
    pub fn read(mut stream: impl Read) -> io::Result<TraceEvent> {
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
                Ok(TraceEvent::Span(SpanEvent::Start(SpanStart {
                    thread_id,
                    time,
                    label_id,
                    extra_data: None,
                })))
            }
            1 => {
                // SpanEnd
                let mut data = [0u8; 16];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());
                Ok(TraceEvent::Span(SpanEvent::End(SpanEnd {
                    thread_id,
                    time,
                })))
            }
            2 => {
                // CountersUpdate
                let mut data = [0u8; 32];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let dropped_packets_delta = u64::from_le_bytes(data[8..16].try_into().unwrap());
                let last_time = u64::from_le_bytes(data[16..24].try_into().unwrap());
                let time = u64::from_le_bytes(data[24..32].try_into().unwrap());
                Ok(TraceEvent::CountersUpdate(CountersUpdate {
                    thread_id,
                    dropped_packets_delta,
                    last_time,
                    time,
                }))
            }
            3 => {
                // SpanStartAdditionalData
                let mut data = [0u8; 32];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());
                let label_id = u64::from_le_bytes(data[16..24].try_into().unwrap());
                let extra_data_length = u64::from_le_bytes(data[24..32].try_into().unwrap());

                let mut extra_data = vec![0u8; extra_data_length as usize];
                stream.read_exact(&mut extra_data)?;

                Ok(TraceEvent::Span(SpanEvent::Start(SpanStart {
                    thread_id,
                    time,
                    label_id,
                    extra_data: Some(extra_data),
                })))
            }
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

fn read_string(stream: &mut impl Read) -> io::Result<String> {
    let length = read_u32(stream)?;
    let mut string_buf = vec![0u8; length as usize];
    stream.read_exact(&mut string_buf)?;
    String::from_utf8(string_buf)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8"))
}

fn read_u32(stream: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}
