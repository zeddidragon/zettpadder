use crate::coords::{Coords};

const SMOOTHING_BUFFER_SIZE: usize = 16;

pub struct Smoothing<T> {
    buffer: [T; SMOOTHING_BUFFER_SIZE],
    idx: usize,
    treshold: f64,
    leeway: f64,
}

impl Smoothing<f64> {
    pub fn radians() -> Self {
        let cutoff = 0.04;
        let treshold = cutoff * 0.5;
        let leeway = cutoff - treshold;
        Self {
            buffer: [0.0; SMOOTHING_BUFFER_SIZE],
            idx: 0,
            treshold: treshold,
            leeway: leeway,
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
            (magnitude - self.treshold) / self.leeway)
            .clamp(0.0, 1.0);

        value * direct_weight + self.smooth(value * (1.0 - direct_weight))
    }

    pub fn clear(&mut self) {
        for v in &mut self.buffer {
            *v *= 0.0;
        }
    }
}

impl Smoothing<Coords> {
    pub fn coords(cutoff: f64) -> Self {
        let treshold = cutoff * 0.5;
        let leeway = cutoff - treshold;
        Self {
            buffer: [Coords::new(); SMOOTHING_BUFFER_SIZE],
            idx: 0,
            treshold: treshold,
            leeway: leeway,
        }
    }

    pub fn smooth(&mut self, value: Coords) -> Coords {
        self.idx = (self.idx + 1) % self.buffer.len();
        self.buffer[self.idx] = value;
        let mut avg = Coords::new();
        for v in &self.buffer {
            avg += *v;
        }
        avg / (self.buffer.len() as f64)
    }

    pub fn tier_smooth(&mut self, value: Coords) -> Coords {
        let magnitude = value.len();

        let direct_weight = 1.0 - (
            (magnitude - self.treshold) / self.leeway)
            .clamp(0.0, 1.0);

        value * direct_weight + self.smooth(value * (1.0 - direct_weight))
    }

    pub fn clear(&mut self) {
        for v in &mut self.buffer {
            *v *= 0.0;
        }
    }
}
