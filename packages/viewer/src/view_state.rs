use crate::proto::{InitialMessage, SpanEnd, SpanEvent, SpanStart};
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

enum ViewRange {
    Full,
    Slice(DisplayPosition),
}

#[derive(Clone, Copy)]
pub struct DisplayPosition {
    pub offset: f64,
    pub cycles_per_pixel: f64,
}

impl DisplayPosition {
    pub fn range(&self, range: f64) -> (f64, f64) {
        (self.offset, self.offset + range * self.cycles_per_pixel)
    }
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
        let selected_span = self.selected_span.as_ref()?;
        let thread = self.threads.get(&selected_span.thread_id)?;
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

    pub fn translate_x(&mut self, cycles_delta: f64, current_position: DisplayPosition) {
        if cycles_delta == 0. {
            return;
        }

        self.view_range = ViewRange::Slice(DisplayPosition {
            offset: current_position.offset + cycles_delta,
            cycles_per_pixel: current_position.cycles_per_pixel,
        });
    }

    pub fn zoom_anchored(
        &mut self,
        multiplier: f64,
        anchor: f64,
        width: f64,
        current_position: DisplayPosition,
    ) {
        if multiplier == 1. {
            return;
        }

        let current_width = width;
        let new_width = width * multiplier;

        let delta = (current_width - new_width) * current_position.cycles_per_pixel;

        self.view_range = ViewRange::Slice(DisplayPosition {
            offset: current_position.offset + (anchor * delta),
            cycles_per_pixel: current_position.cycles_per_pixel * multiplier,
        });
    }

    pub fn position(&self, range: f64) -> DisplayPosition {
        match self.view_range {
            ViewRange::Full => {
                let (min, max) = self
                    .timestamp_range
                    .values
                    .unwrap_or((0, self.initial_message.tsc_frequency));

                let offset = min as f64;

                let delta = max as f64 - min as f64;

                let cycles_per_pixel = delta / range;

                DisplayPosition {
                    offset,
                    cycles_per_pixel,
                }
            }
            ViewRange::Slice(position) => position,
        }
    }
}
