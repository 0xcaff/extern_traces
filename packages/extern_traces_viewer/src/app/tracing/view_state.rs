use crate::app::tracing::timeline_position::TimelinePositionState;
use crate::proto::{InitialMessage, SpanEnd, SpanEvent, SpanStart};
use std::collections::BTreeMap;
use std::ops::Range;

#[derive(Clone)]
pub struct ThreadSpan {
    pub start_time: u64,
    pub end_time: u64,
    pub label_id: u64,
    pub start_extra_data: Option<Vec<u8>>,
    pub end_extra_data: Option<Vec<u8>>,
}

impl ThreadSpan {
    pub fn from_events(start: SpanStart, end: SpanEnd) -> ThreadSpan {
        ThreadSpan {
            start_time: start.time,
            end_time: end.time,
            label_id: start.label_id,
            start_extra_data: start.extra_data,
            end_extra_data: end.extra_data,
        }
    }
}

pub struct ThreadState {
    pub spans: Vec<ThreadSpan>,
    pub currently_started: Option<SpanStart>,
    pub folded_spans_state: FoldSpansState,
}

pub struct FoldSpansState {
    last_calculated_value: Option<(Range<usize>, Vec<(usize, usize)>)>,
}

impl FoldSpansState {
    pub fn new() -> FoldSpansState {
        FoldSpansState {
            last_calculated_value: None,
        }
    }
}

impl FoldSpansState {
    pub fn fold(
        &mut self,
        range: Range<usize>,
        thread_spans: &[ThreadSpan],
        min_cycles: u64,
    ) -> &[(usize, usize)] {
        {
            if self.last_calculated_value.is_some() {
                let (last_range, _value) = self.last_calculated_value.as_ref().unwrap();

                if last_range == &range {
                    let (_last_range, value) = self.last_calculated_value.as_ref().unwrap();
                    return value;
                }
            }
        }

        let mut spans = Vec::new();

        let mut idx = range.start;
        while idx < range.end {
            let span = &thread_spans[idx];

            let mut next_idx = idx + 1;
            loop {
                if next_idx >= range.end {
                    break;
                }

                let next_span = &thread_spans[next_idx];
                if (next_span.end_time - span.start_time) >= min_cycles {
                    break;
                }

                next_idx += 1;
            }

            spans.push((idx, next_idx));
            idx = next_idx;
        }

        self.last_calculated_value = Some((range, spans));

        self.last_calculated_value.as_ref().unwrap().1.as_slice()
    }
}

#[derive(Clone)]
pub struct SpanRef {
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
    pub selected_span: Option<SpanRef>,
    pub current_symbol_detail: Option<usize>,
    pub extra_data_messages: Vec<SpanRef>,

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
            current_symbol_detail: None,
            extra_data_messages: vec![],
        })
    }
}

impl ViewState {
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
                        folded_spans_state: FoldSpansState::new(),
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

                    let span_ref = SpanRef {
                        span_idx: state.spans.len(),
                        thread_id: end.thread_id,
                    };
                    let span = ThreadSpan::from_events(start, end);
                    if span.start_extra_data.is_some() || span.end_extra_data.is_some() {
                        self.extra_data_messages.push(span_ref);
                    }

                    state.spans.push(span);
                };
            }
        }
    }

    pub fn total_spans(&self) -> usize {
        self.threads.values().map(|v| v.spans.len()).sum()
    }
}
