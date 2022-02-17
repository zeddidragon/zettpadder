use rdev; use crossbeam_channel::{Receiver, tick};
use std::collections::{HashMap};
use super::mapping::{Binding, Mapping};
use std::time::Duration;
use std::f64::consts::{PI, TAU};

const LAYER_SIZE: u16 = 256;

use crate::coords::{Coords};
use crate::function::{Function};
use crate::smoothing::{Smoothing};

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

const FPS: u64 = 60;  // Default loop rate
const FLICK_FACTOR: f64 = 1280.0;  // How much one radian moves the mouse
const FLICK_DEADZONE: f64 = 0.9; // Deadzone to engage flick
const FLICK_TIME: Duration = Duration::from_millis(100); // Duration of a flick

#[derive(Debug, Copy, Clone)]
pub enum ZpMsg {
    Input(u8, f64), // Input from controller to process
    SetWriteLayer(u8), // Layer used in future assignments
    SetFps(u64), // Cycle rate of main loop
    SetFlickFactor(f64), // Mouse motion of one radian
    SetFlickTime(u64), // Duration of a flick
    SetFlickDeadzone(f64), // Deadzone of stick before initiating flick
    Bind(u8, Mapping), // Bind output to button
    BindFunction(u8, Function), // Bind function to button
    SetDeadzoneOn(u8, f64), // Deadzone before binding enables
    SetDeadzoneOff(u8, f64), // Deadzone before binding disables
}

pub fn run(receiver: Receiver<ZpMsg>) {
    let mut tick_time = Duration::from_nanos(1_000_000_000 / FPS);
    let mut ticker = tick(tick_time);
    let mut layer = 0;
    let mut write_layer = 0;
    let mut mover = Coords::new();
    let mut motion = Coords::new();
    let mut released_layers = Vec::with_capacity(8);
    let mut keymaps: HashMap<u16, Binding> = HashMap::new();
    let mut values: HashMap<u8, f64> = HashMap::new();
    let mut functions: Vec<Function> = Vec::new();

    let mut flicker = Coords::new();
    let mut prev_flicker;
    let mut flick_deadzone = FLICK_DEADZONE;
    let mut flick_smoother = Smoothing::new();
    let mut total_flick_steering = 0.0;
    let mut flick_time = FLICK_TIME;
    let mut flick_remaining = Duration::ZERO;
    let mut flick_tick = 0.0;
    let mut flick_factor = FLICK_FACTOR;

    loop {
        motion *= 0.0;
        ticker.recv().unwrap();
        prev_flicker = flicker;
        while let Ok(msg) = receiver.try_recv() {
            use ZpMsg::*;
            match msg {
                Input(id, value) => {
                    let idx = id as u16;
                    let shifted = idx + LAYER_SIZE * (layer as u16);
                    let binding =
                        if let Some(m) = keymaps.get(&shifted) {
                            Some(m)
                        } else if let Some(m) = keymaps.get(&idx) {
                            Some(m)
                        } else {
                            None
                        };
                    if let Some(binding) = binding {
                        let prev = values.entry(id).or_default();
                        let mapping = binding.get_mapping(value, *prev);
                        *prev = value;
                        match mapping {
                            Some(Mapping::Layer(l)) => {
                                if l != layer {
                                    // Untrigger any chorded keys
                                    if layer > 0 {
                                        released_layers.push(layer);
                                    }
                                    layer = l;
                                }
                            },
                            Some(Mapping::MouseX(v)) => {
                                mover.x = v * value;
                            },
                            Some(Mapping::MouseY(v)) => {
                                mover.y = v * value;
                            },
                            Some(Mapping::FlickX) => {
                                flicker.x = value;
                            },
                            Some(Mapping::FlickY) => {
                                flicker.y = value;
                            },
                            Some(Mapping::Emit(ev)) => {
                                send(&ev);
                            },
                            Some(Mapping::Trigger(idx)) => {
                                functions[idx].value = value;
                            },
                            Some(Mapping::Noop) => {},
                            None => {},
                            unx => {
                                println!("Received: {:?}, which is unexpected", unx);
                            }
                        };
                    }
                },
                Bind(button, mapping) => {
                    let idx = button as u16 + write_layer as u16 * 256;
                    let mut binding = Binding::new(mapping);
                    match mapping {
                        Mapping::FlickX => { binding.deadzone_on = Some(0.0); },
                        Mapping::FlickY => { binding.deadzone_on = Some(0.0); },
                        _ => {},
                    }
                    keymaps.insert(idx, binding);
                },
                BindFunction(button, function) => {
                    let idx = functions.len();
                    functions.push(function);
                    let mapping = Mapping::Trigger(idx);
                    let idx = button as u16 + write_layer as u16 * 256;
                    keymaps.insert(idx, Binding::new(mapping));
                },
                SetDeadzoneOn(button, v) => {
                    let idx = button as u16 + write_layer as u16 * 256;
                    if let Some(binding) = keymaps.get_mut(&idx) {
                        binding.deadzone_on = Some(v);
                    } else {
                        println!("No binding found for {} ({})", button, idx);
                    }
                },
                SetDeadzoneOff(button, v) => {
                    let idx = button as u16 + write_layer as u16 * 256;
                    if let Some(binding) = keymaps.get_mut(&idx) {
                        binding.deadzone_off = Some(v);
                    } else {
                        println!("No binding found for {} ({})", button, idx);
                    }
                },

                SetWriteLayer(v) => { write_layer = v; },
                SetFps(v) => {
                    tick_time = Duration::from_nanos(1_000_000_000 / v);
                    ticker = tick(tick_time);
                },
                SetFlickFactor(v) => { flick_factor = v; },
                SetFlickTime(v) => { flick_time = Duration::from_millis(v); },
                SetFlickDeadzone(v) => { flick_deadzone = v; },
            }
        }

        // Release any chorded presses when chord is released
        if !released_layers.is_empty() {
            for l in &released_layers {
                let range = (LAYER_SIZE * *l as u16)..(LAYER_SIZE * (*l as u16 + 1));
                for (k, binding) in keymaps.iter_mut() {
                    if !range.contains(k) { continue; }
                    let idx = *k as u8;
                    let prev = values.entry(idx).or_default();
                    if *prev == 0.0 { continue; }
                    let release = binding.get_mapping(0.0, *prev);
                    match release {
                        Some(Mapping::Emit(ev)) => {
                            send(&ev);
                        },
                        Some(Mapping::Trigger(idx)) => {
                            functions[idx].value = 0.0;
                        },
                        _ => {},
                    };
                }
            }
            released_layers.clear();
        }

        // Old school moving
        if mover.len() > 0.0 {
            motion = mover;
        }

        // Flick sticking
        if flicker.len() >= flick_deadzone {
            if prev_flicker.len() < flick_deadzone {
                // Starting a flick
                let angle = flicker.angle();
                let ticks = ( flick_time.as_nanos()
                    / tick_time.as_nanos()) as f64;
                flick_remaining = flick_time;
                flick_tick = flick_factor * angle / ticks;
                flick_smoother.clear();
                total_flick_steering = 0.0;

            } else {
                // Steering
                let angle = flicker.angle();
                let prev_angle = prev_flicker.angle();
                let diff = angle - prev_angle;
                let diff = modulo(diff + PI, TAU) - PI;
                total_flick_steering += diff;
                let diff = flick_smoother.tier_smooth(diff);
                motion.x += flick_factor * diff;
            }
        }

        if flick_remaining > Duration::ZERO {
            flick_remaining =
                if flick_remaining > tick_time {
                    flick_remaining - tick_time
                } else { Duration::ZERO };
            motion.x += flick_tick;
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
