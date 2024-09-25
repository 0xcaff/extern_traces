use crate::proto::InitialMessage;
use std::time::SystemTime;

pub struct TimelinePositionState {
    tsc_frequency: u64,

    anchor_time: SystemTime,
    anchor_timestamp: u64,

    is_live: bool,
    view_range: ViewRange,
    timestamp_range: TimestampRange,
}

impl TimelinePositionState {
    pub fn new(initial_message: &InitialMessage) -> TimelinePositionState {
        TimelinePositionState {
            tsc_frequency: initial_message.tsc_frequency,
            anchor_time: initial_message.anchor(),
            anchor_timestamp: initial_message.anchor_timestamp,
            is_live: false,
            view_range: ViewRange::Full,
            timestamp_range: TimestampRange { values: None },
        }
    }

    pub fn add_timestamp_range(&mut self, timestamp: u64) {
        self.timestamp_range.add_value(timestamp);
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
        range: f64,
        current_position: DisplayPosition,
    ) {
        if multiplier == 1. {
            return;
        }

        let delta = (1. - multiplier) * range * current_position.cycles_per_pixel;

        self.view_range = ViewRange::Slice(DisplayPosition {
            offset: current_position.offset + (anchor * delta),
            cycles_per_pixel: current_position.cycles_per_pixel * multiplier,
        });
    }

    pub fn position(&self, range: f64) -> DisplayPosition {
        if let ViewRange::Slice(position) = self.view_range {
            return position;
        }

        let (min, max) = self.timestamp_range.values.unwrap_or((0, 0));

        let offset = min as f64;
        let max = if self.is_live {
            let duration = self.anchor_time.elapsed().ok().unwrap();
            duration.as_secs_f64() * self.tsc_frequency as f64
        } else {
            max as f64
        };

        let delta = max - min as f64;

        let cycles_per_pixel = delta / range;

        DisplayPosition {
            offset,
            cycles_per_pixel,
        }
    }
}

struct TimestampRange {
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
