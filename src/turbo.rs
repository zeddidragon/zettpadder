// Turbo currently in pressed state
const TURBO_STATE_PRESSED: u8 = 0x1;

pub const TURBO_CYCLE: u8 = 20;

use crate::mapping::{Mapping};

#[derive(Debug, Copy, Clone)]
pub struct Turbo {
    pub rate: f64,
    state: u8,
    press_mapping: Option<Mapping>,
    release_mapping: Option<Mapping>,
}

impl Turbo {
    pub fn new(press: Option<Mapping>, release: Option<Mapping>) -> Self {
        Self {
            rate: 0.0,
            state: 0,
            press_mapping: press,
            release_mapping: release,
        }
    }

    #[inline]
    pub fn check(&mut self, tick: u8) -> Option<Mapping> {
        let ratio = ((TURBO_CYCLE as f64) * self.rate).round() as u8;
        if ratio > tick {
            self.press()
        } else if ratio <= tick {
            self.release()
        } else {
            None
        }
    }

    #[inline]
    pub fn is_pressed(&mut self) -> bool {
        self.state & TURBO_STATE_PRESSED == TURBO_STATE_PRESSED
    }

    #[inline]
    pub fn press(&mut self) -> Option<Mapping> {
        if self.is_pressed() {
            None
        } else {
            self.state = self.state | TURBO_STATE_PRESSED;
            self.press_mapping
        }
    }

    #[inline]
    pub fn release(&mut self) -> Option<Mapping> {
        if self.is_pressed() {
            self.state = self.state & !TURBO_STATE_PRESSED;
            self.release_mapping
        } else {
            None
        }
    }
}
