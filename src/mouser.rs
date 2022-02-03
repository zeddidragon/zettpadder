use rdev;
use crossbeam_channel::{Receiver, tick};
use super::state::{MouseMsg, MouseMsgType};
use std::time::Duration;
use std::f64::consts::{PI, TAU};
use std::ops;

const FLICK_TIME : Duration = Duration::from_millis(100);
const FLICK_DEADZONE : f64 = 0.9;

const MOVE_DEADZONE : f64 = 0.1;
const MOVE_MULTIPLIER : f64 = 10.0;

const FPS : u64 = 240;
const TICK_TIME : Duration = Duration::from_nanos(1_000_000_000 / FPS);

#[derive(Debug, Copy, Clone)]
struct Coords { x: f64,
    y: f64,
}

impl Coords {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }

    fn len(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2).sqrt()
    }

    fn manhattan(&self) -> f64 {
        self.x.abs() + self.y.abs()
    }

    fn angle(&self) -> f64 {
        self.x.atan2(-self.y)
    }
}

impl ops::Add<Coords> for Coords {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Coords {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::AddAssign<Coords> for Coords {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl ops::Mul<f64> for Coords {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Coords {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::MulAssign<f64> for Coords {
    fn mul_assign(&mut self, other: f64) {
        self.x *= other;
        self.y *= other;
    }
}

#[inline]
fn modulo(v: f64, k: f64) -> f64 {
    v - (v / k).floor() * k
}

pub struct Mouser {
    mover: Coords,
    flicker: Coords,
    prev_flicker: Coords,
    flick_remaining: Duration,
    flick_tick: f64,
    pub receiver: Receiver<MouseMsg>,
}

impl Mouser {
    pub fn new(
        receiver: Receiver<MouseMsg>,
    ) -> Self {
        Self {
            mover: Coords::new(),
            flicker: Coords::new(),
            prev_flicker: Coords::new(),
            flick_remaining: Duration::ZERO,
            flick_tick: 0.0,
            receiver: receiver,
        }
    }

    pub fn run(&mut self) {
        let ticker = tick(TICK_TIME);
        let mut motion = Coords::new();
        loop {
            motion *= 0.0;
            ticker.recv().unwrap();
            self.prev_flicker = self.flicker;
            while let Ok((msg_type, value)) = self.receiver.try_recv() {
                match msg_type {
                    MouseMsgType::MoveX => {
                        self.mover.x = value;
                    },
                    MouseMsgType::MoveY => {
                        self.mover.y = value;
                    },
                    MouseMsgType::FlickX => {
                        self.flicker.x = value;
                    },
                    MouseMsgType::FlickY => {
                        self.flicker.y = value;
                    },
                }
            }

            // Old school moving
            if self.mover.len() > MOVE_DEADZONE {
                motion = self.mover * MOVE_MULTIPLIER;
            }

            // TODO: Configure this
            let full_flick = 256.0 * 12.0;
            // Flick sticking
            if self.flicker.len() >= FLICK_DEADZONE {
                if self.prev_flicker.len() < FLICK_DEADZONE {
                    // Starting a flick
                    let angle = self.flicker.angle();
                    let ticks = (FLICK_TIME.as_nanos() / TICK_TIME.as_nanos())
                        as f64;
                    self.flick_remaining = FLICK_TIME;
                    self.flick_tick = full_flick * angle / PI / ticks;

                } else {
                    // Steering
                    let angle = self.flicker.angle();
                    let prev_angle = self.prev_flicker.angle();
                    let diff = angle - prev_angle;
                    let diff = modulo(diff + PI, TAU) - PI;
                    motion.x += full_flick * diff / PI;
                }
            }

            if self.flick_remaining > Duration::ZERO {
                self.flick_remaining =
                    if self.flick_remaining > TICK_TIME {
                        self.flick_remaining - TICK_TIME
                    } else { Duration::ZERO };
                motion.x += self.flick_tick;
            }

            // Apply all motion in the tick
            if motion.manhattan() > 0.0 {
                let event = rdev::EventType::MouseMoveRelative {
                    delta_x: motion.x,
                    delta_y: motion.y,
                };
                match rdev::simulate(&event) {
                    Ok(()) => (),
                    Err(rdev::SimulateError) => {
                        println!("Unable to can {:?}", event);
                    }
                }
            }
        }
    }
}
