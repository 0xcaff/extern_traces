use crate::proto::{InitialMessage, SpanEnd, SpanEvent, SpanStart};
use crate::timeline_position::TimelinePositionState;
use std::collections::BTreeMap;

#[derive(Clone)]
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

pub fn fold_spans(thread_spans: &[ThreadSpan], cycles_per_pixel: u64) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();

    let mut idx = 0;
    while idx < thread_spans.len() {
        let span = &thread_spans[idx];

        let mut next_idx = idx + 1;
        loop {
            if next_idx >= thread_spans.len() {
                break;
            };

            let next_span = &thread_spans[next_idx];
            if (next_span.end_time - span.start_time) >= cycles_per_pixel {
                break;
            }

            next_idx = next_idx + 1;
        }

        spans.push((idx, next_idx));
        idx = next_idx;
    }

    spans
}

pub struct SelectedSpanMetadata {
    pub thread_id: u64,
    pub span_idx: usize,
}

pub enum ViewStateContainer {
    Empty,
    Initialized(ViewState),
}

pub struct ViewState {
    pub initial_message: InitialMessage,
    pub threads: BTreeMap<u64, ThreadState>,
    pub selected_span: Option<SelectedSpanMetadata>,
    pub is_live: bool,
    pub current_symbol_detail: Option<usize>,

    pub timeline_position_state: TimelinePositionState,
}

impl ViewStateContainer {
    pub fn new() -> ViewStateContainer {
        ViewStateContainer::Empty
    }

    pub fn initialize(&mut self, initial_message: InitialMessage) {
        *self = ViewStateContainer::Initialized(ViewState {
            timeline_position_state: TimelinePositionState::new(&initial_message),
            initial_message,
            threads: BTreeMap::new(),
            selected_span: None,
            is_live: false,
            current_symbol_detail: None,
        })
    }
}

impl ViewState {
    pub fn selected_span_ref(&self) -> Option<(&ThreadState, &ThreadSpan)> {
        let selected_span = self.selected_span.as_ref()?;
        let thread = self.threads.get(&selected_span.thread_id)?;
        let span = &thread.spans[selected_span.span_idx];

        Some((thread, span))
    }

    pub fn update_span(&mut self, event: SpanEvent) {
        match event {
            SpanEvent::Start(start) => {
                self.timeline_position_state.add_timestamp_range(start.time);

                let thread = self
                    .threads
                    .entry(start.thread_id)
                    .or_insert_with(|| ThreadState {
                        spans: Vec::new(),
                        currently_started: None,
                    });

                if let Some(..) = thread.currently_started.replace(start) {
                    println!("two spans started");
                };
            }
            SpanEvent::End(end) => {
                self.timeline_position_state.add_timestamp_range(end.time);

                let state = self.threads.get_mut(&end.thread_id).unwrap();
                if let Some(start) = state.currently_started.take() {
                    if let Some(last_span) = state.spans.last() {
                        if start.time < last_span.end_time {
                            println!(
                                "out-of-order span detected on thread {}: start time {} is before last span end time {}",
                                end.thread_id,
                                start.time,
                                last_span.end_time
                            );

                            return;
                        }
                    }

                    state.spans.push(ThreadSpan::from_events(start, end));
                };
            }
        }
    }

    pub fn total_spans(&self) -> usize {
        self.threads.values().map(|v| v.spans.len()).sum()
    }
}
