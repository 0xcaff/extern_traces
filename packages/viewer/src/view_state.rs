use crate::proto::{InitialMessage, SpanEnd, SpanEvent, SpanStart, TraceEvent};
use std::collections::BTreeMap;

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

pub fn fold_spans(thread_spans: &[&ThreadSpan], cycles_per_pixel: u64) -> Vec<(usize, usize)> {
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

        spans.push(((idx, next_idx)));
        idx = next_idx;
    }

    spans
}

pub enum ViewRange {
    Full,
    Slice((u64, u64)),
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
    pub timestamp_range: TimestampRange,
    pub view_range: ViewRange,
    pub selected_span: Option<SelectedSpanMetadata>,
    pub is_live: bool,
}

impl ViewStateContainer {
    pub fn new() -> ViewStateContainer {
        ViewStateContainer::Empty
    }

    pub fn initialize(&mut self, initial_message: InitialMessage) {
        *self = ViewStateContainer::Initialized(ViewState {
            initial_message,
            threads: BTreeMap::new(),
            timestamp_range: TimestampRange { values: None },
            view_range: ViewRange::Full,
            selected_span: None,
            is_live: false,
        })
    }
}

#[derive(Debug)]
pub struct TimestampRange {
    values: Option<(u64, u64)>,
}

impl TimestampRange {
    pub fn add_value(&mut self, value: u64) {
        let old_values = self.values.take();
        let (low, high) = if let Some((low, high)) = old_values {
            (u64::min(low, value), u64::max(high, value))
        } else {
            (value, value)
        };

        self.values.replace((low, high));
    }
}

impl ViewState {
    pub fn selected_span_ref(&self) -> Option<(&ThreadState, &ThreadSpan)> {
        let Some(selected_span) = &self.selected_span else {
            return None;
        };

        let thread = self.threads.get(&selected_span.thread_id).unwrap();
        let span = &thread.spans[selected_span.span_idx];

        Some((thread, span))
    }

    pub fn update_span(&mut self, event: SpanEvent) {
        match event {
            SpanEvent::Start(start) => {
                self.timestamp_range.add_value(start.time);

                let thread = self
                    .threads
                    .entry(start.thread_id)
                    .or_insert_with(|| ThreadState {
                        spans: Vec::new(),
                        currently_started: None,
                    });
                if let None = thread.currently_started.replace(start) {
                    // println!("two spans started");
                };
            }
            SpanEvent::End(end) => {
                self.timestamp_range.add_value(end.time);

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

    pub fn translate_x(&mut self, value: i64) {
        if value == 0 {
            return;
        }

        let (min, max) = self.range();
        self.view_range = ViewRange::Slice((combine(min, value), combine(max, value)));
    }

    pub fn zoom_anchored(&mut self, multiplier: f32, anchor: f32) {
        if multiplier == 1.0f32 {
            return;
        }

        let (min, max) = self.range();

        let original_size = max - min;
        let new_size = original_size as f32 * multiplier;
        let original_anchor = ((anchor * original_size as f32) as u64).saturating_add(min);

        let new_min = original_anchor.saturating_sub((anchor * new_size) as _);
        let new_max = original_anchor.saturating_add(((1.0 - anchor) * new_size) as _);

        self.view_range = ViewRange::Slice((new_min, new_max));
    }

    pub fn range(&self) -> (u64, u64) {
        match self.view_range {
            ViewRange::Full => self.timestamp_range.values.unwrap_or((0, 1_000_000_000)),
            ViewRange::Slice(values) => values,
        }
    }
}

fn combine(base: u64, signed: i64) -> u64 {
    if signed > 0 {
        base.saturating_add(signed as u64)
    } else {
        base.saturating_sub(-signed as u64)
    }
}
