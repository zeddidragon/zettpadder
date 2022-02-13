use rdev; use crossbeam_channel::{Receiver, tick};
use super::controller_poller::{Message};
use std::collections::{BTreeMap};
use super::mapping::{Binding, Mapping};
use std::time::Duration;
use std::f64::consts::{PI, TAU};

const LAYER_SIZE: u16 = 256;

use crate::coords::{Coords};
use crate::function::{Function};

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

pub struct ZettpadderConfig {
    pub fps: u64,
    pub flick_180: f64,
    pub flick_time: u64,
    pub flick_deadzone: f64,
    pub move_deadzone: f64,
    pub move_multiplier: f64,
}

impl ZettpadderConfig {
    pub fn new() -> Self {
        Self {
            fps: 60,
            flick_180: 2048.0,
            flick_time: 100,
            flick_deadzone: 0.9,
            move_deadzone: 0.1,
            move_multiplier: 10.0
        }
    }
}

pub struct Zettpadder {
    tick_time: Duration,
    flick_180: f64,
    flick_time: Duration,
    flick_deadzone: f64,
    move_deadzone: f64,
    move_multiplier: f64,
    keymaps: BTreeMap<u16, Binding>,
    values: BTreeMap<u8, f64>, // Values of the buttons
    functions: Vec<Function>,
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
        config: ZettpadderConfig,
        receiver: Receiver<Message>,
        keymaps: BTreeMap<u16, Binding>,
        functions: Vec<Function>,
    ) -> Self {
        let mut values = BTreeMap::new();
        for (key, _) in &keymaps {
            values.insert(*key as u8, 0.0);
        }
        Self {
            tick_time: Duration::from_nanos(1_000_000_000 / config.fps),
            flick_180: config.flick_180,
            flick_time: Duration::from_millis(config.flick_time),
            flick_deadzone: config.flick_deadzone,
            move_deadzone: config.move_deadzone,
            move_multiplier: config.move_multiplier,
            keymaps: keymaps,
            values: values,
            functions: functions,
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
        let ticker = tick(self.tick_time);
        let mut motion = Coords::new();
        let mut released_layers = Vec::with_capacity(8);
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
                                if self.layer > 0 {
                                    released_layers.push(self.layer);
                                }
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
                        Some(Mapping::Trigger(idx)) => {
                            self.functions[idx].value = value;
                        },
                        None => {},
                        unx => {
                            println!("Received: {:?}, which is unexpected", unx);
                        }
                    };
                }
            }

            if !released_layers.is_empty() {
                for l in &released_layers {
                    let range = (LAYER_SIZE * *l as u16)..(LAYER_SIZE * (*l as u16 + 1));
                    for (k, binding) in self.keymaps.iter_mut() {
                        if !range.contains(k) { continue; }
                        let idx = &(*k as u8);
                        let prev = self.values[idx];
                        if prev == 0.0 { continue; }
                        let release = binding.get_mapping(0.0, prev);
                        match release {
                            Some(Mapping::Emit(ev)) => {
                                send(&ev);
                            },
                            Some(Mapping::Trigger(idx)) => {
                                self.functions[idx].value = 0.0;
                            },
                            _ => {},
                        };
                    }
                }
                released_layers.clear();
            }

            // Old school moving
            if self.mover.len() > self.move_deadzone {
                motion = self.mover * self.move_multiplier;
            }

            // Flick sticking
            if self.flicker.len() >= self.flick_deadzone {
                if self.prev_flicker.len() < self.flick_deadzone {
                    // Starting a flick
                    let angle = self.flicker.angle();
                    let ticks = ( self.flick_time.as_nanos()
                        / self.tick_time.as_nanos()) as f64;
                    self.flick_remaining = self.flick_time;
                    self.flick_tick = self.flick_180 * angle / PI / ticks;

                } else {
                    // Steering
                    let angle = self.flicker.angle();
                    let prev_angle = self.prev_flicker.angle();
                    let diff = angle - prev_angle;
                    let diff = modulo(diff + PI, TAU) - PI;
                    motion.x += self.flick_180 * diff / PI;
                }
            }

            if self.flick_remaining > Duration::ZERO {
                self.flick_remaining =
                    if self.flick_remaining > self.tick_time {
                        self.flick_remaining - self.tick_time
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
