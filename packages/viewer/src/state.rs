use crate::proto::{InitialMessage, SpanEnd, SpanEvent, SpanStart, TraceEvent};
use std::collections::HashMap;

pub struct ThreadSpan {
    pub start_time: u64,
    pub end_time: u64,
    pub label_id: u64,
}

impl ThreadSpan {
    pub fn from_events(start: SpanStart, end: SpanEnd) -> ThreadSpan {
        ThreadSpan {
            start_time: start.time,
            end_time: end.time,
            label_id: start.label_id,
        }
    }
}

pub struct ThreadState {
    pub spans: Vec<ThreadSpan>,
    pub currently_started: Option<SpanStart>,
}

pub struct State {
    pub initial_message: Option<InitialMessage>,
    pub threads: HashMap<u64, ThreadState>,
}

impl State {
    pub fn new() -> State {
        State {
            initial_message: None,
            threads: HashMap::new(),
        }
    }
}

impl State {
    pub fn update_trace(&mut self, event: TraceEvent) {
        match event {
            TraceEvent::Start(initial_message) => {
                self.initial_message.replace(initial_message);
            }
            TraceEvent::Span(span) => self.update_span(span),
        }
    }

    pub fn update_span(&mut self, event: SpanEvent) {
        match event {
            SpanEvent::Start(start) => {
                let thread = self
                    .threads
                    .entry(start.thread_id)
                    .or_insert_with(|| ThreadState {
                        spans: Vec::new(),
                        currently_started: None,
                    });
                if let None = thread.currently_started.replace(start) {
                    println!("two spans started");
                };
            }
            SpanEvent::End(end) => {
                let state = self.threads.get_mut(&end.thread_id).unwrap();
                if let Some(start) = state.currently_started.take() {
                    state.spans.push(ThreadSpan::from_events(start, end));
                };
            }
        }
    }

    pub fn total_spans(&self) -> usize {
        self.threads.values().map(|v| v.spans.len()).sum()
    }
}
