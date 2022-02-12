use rdev;
use crossbeam_channel::{Receiver, tick};
use super::controller_poller::{Message};
use std::collections::{BTreeMap};
use super::mapping::{Binding, Mapping};
use std::time::Duration;
use std::f64::consts::{PI, TAU};

const FLICK_TIME: Duration = Duration::from_millis(100);
const FLICK_DEADZONE: f64 = 0.9;

const MOVE_DEADZONE: f64 = 0.1;
const MOVE_MULTIPLIER: f64 = 10.0;

const FPS: u64 = 60;
const TICK_TIME: Duration = Duration::from_nanos(1_000_000_000 / FPS);

const LAYER_SIZE: u16 = 256;

use crate::coords::{Coords};
use crate::turbo::{Turbo};

#[inline]
fn modulo(v: f64, k: f64) -> f64 {
    v - (v / k).floor() * k
}

fn send(event_type: &rdev::EventType) {
    match rdev::simulate(event_type) {
        Ok(()) => (),
        Err(rdev::SimulateError) => {
            println!("Unable to can {:?}", event_type);
        }
    }
}

pub struct Zettpadder {
    keymaps: BTreeMap<u16, Binding>,
    values: BTreeMap<u8, f64>, // Values of the buttons
    turbos: Vec<Turbo>,
    layer: u8,
    mover: Coords,
    flicker: Coords,
    prev_flicker: Coords,
    flick_remaining: Duration,
    flick_tick: f64,
    pub receiver: Receiver<Message>,
}

impl Zettpadder {
    pub fn new(
        receiver: Receiver<Message>,
        keymaps: BTreeMap<u16, Binding>,
        turbos: Vec<Turbo>,
    ) -> Self {
        let mut values = BTreeMap::new();
        for (key, _) in &keymaps {
            values.insert(*key as u8, 0.0);
        }
        Self {
            keymaps: keymaps,
            values: values,
            turbos: turbos,
            layer: 0,
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
            while let Ok((id, value)) = self.receiver.try_recv() {
                let idx = id as u16;
                let shifted = idx + LAYER_SIZE * (self.layer as u16);
                let binding =
                    if let Some(m) = self.keymaps.get(&shifted) {
                        Some(m)
                    } else if let Some(m) = self.keymaps.get(&idx) {
                        Some(m)
                    } else {
                        None
                    };
                if let Some(binding) = binding {
                    let prev = self.values[&id];
                    self.values.insert(id, value);
                    let mapping = binding.get_mapping(value, prev);
                    match mapping {
                        Some(Mapping::Layer(l)) => {
                            if l != self.layer {
                                // Untrigger any chorded keys
                                self.layer = l;
                            }
                        },
                        Some(Mapping::MouseX(v)) => {
                            self.mover.x = v * value;
                        },
                        Some(Mapping::MouseY(v)) => {
                            self.mover.y = v * value;
                        },
                        Some(Mapping::FlickX) => {
                            self.flicker.x = value;
                        },
                        Some(Mapping::FlickY) => {
                            self.flicker.y = value;
                        },
                        Some(Mapping::Emit(ev)) => {
                            send(&ev);
                        },
                        Some(Mapping::NegPosTurbo(nidx, pidx)) => {
                            self.turbos[nidx].rate = (-value).max(0.0);
                            self.turbos[pidx].rate = value.max(0.0);
                        },
                        None => {},
                        unx => {
                            println!("Received: {:?}, which is unexpected", unx);
                        }
                    };
                }
            }

            // Old school moving
            if self.mover.len() > MOVE_DEADZONE {
                motion = self.mover * MOVE_MULTIPLIER;
            }

            // TODO: Configure this
            let full_flick = 256.0 * 6.0;
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
