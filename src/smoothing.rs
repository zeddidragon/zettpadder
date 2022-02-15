use std::ops::{AddAssign, MulAssign, Div};
use std::default::Default;

pub struct Smoothing<T> {
    buffer: Vec<T>,
    idx: usize,
}

const SMOOTHING_CUTOFF: f64 = 0.04;
const SMOOTHING_TRESHOLD: f64 = SMOOTHING_CUTOFF * 0.5;
const SMOOTHING_LEEWAY: f64 = SMOOTHING_CUTOFF - SMOOTHING_TRESHOLD;
const SMOOTHING_BUFFER_SIZE: usize = 16;

impl Smoothing<f64> {
    pub fn new() -> Self {
        Self {
            buffer: vec!(0.0; SMOOTHING_BUFFER_SIZE),
            idx: 0,
        }
    }

    pub fn smooth(&mut self, value: f64) -> f64 {
        self.idx = (self.idx + 1) % self.buffer.len();
        self.buffer[self.idx] = value;
        let mut avg = 0.0;
        for v in &self.buffer {
            avg += v;
        }
        avg / (self.buffer.len() as f64)
    }

    pub fn tier_smooth(&mut self, value: f64) -> f64 {
        let magnitude = value.abs();

        let direct_weight = 1.0 - (
            (magnitude - SMOOTHING_TRESHOLD) / SMOOTHING_LEEWAY)
            .clamp(0.0, 1.0);

        value * direct_weight + self.smooth(value * (1.0 - direct_weight))
    }

    pub fn clear(&mut self) {
        for v in self.buffer.iter_mut() {
            *v *= 0.0;
        }
    }
}
