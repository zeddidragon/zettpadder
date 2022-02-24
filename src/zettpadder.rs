use rdev;
use stick;
use crossbeam_channel::{Sender, Receiver};
use std::collections::{HashMap};
use crate::mapping::{Binding, Mapping};
use crate::mouser::{MouserMsg};
use crate::macros::{MacroMsg, MacroType};
use crate::cli::{CliMsg};

const LAYER_SIZE: u16 = 256;

fn send(event_type: &rdev::EventType) {
    match rdev::simulate(event_type) {
        Ok(()) => (),
        Err(rdev::SimulateError) => {
            println!("Unable to can {:?}", event_type);
        }
    }
}

fn send_to_mouse(sender: &Sender<MouserMsg>, msg: MouserMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to send to mouse: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

fn send_to_macro(sender: &Sender<MacroMsg>, msg: MacroMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to send to macro: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

fn send_to_cli(sender: &Sender<CliMsg>, msg: CliMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to send to cli: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

#[derive(Debug, Copy, Clone)]
pub enum ZpMsg {
    Output(rdev::EventType), // Perform output directly
    Input(stick::Event), // Input from controller to process mapping for
    SetWriteLayer(u8), // Layer used in future assignments
    SetFps(u64), // Cycle rate of main loop
    SetFlickFactor(f64), // Mouse motion of one radian
    SetFlickTime(u64), // Duration of a flick
    SetFlickDeadzone(f64), // Deadzone before performing a flick
    Bind(u8, Mapping), // Bind output to button
    SetDeadzoneOn(u8, f64), // Deadzone before binding enables
    SetDeadzoneOff(u8, f64), // Deadzone before binding disables
    GetFlickCalibration(f64), // Display data to help calibrate flick factor
    SetEcho(bool), // Echo mode, which repeats your keys back
    CreateMacro(u8, MacroType), // Request to create a macro, assign to this button
    AddToMacro(Mapping), // Add Mapping to currently constructed macro
    MacroCreated(u16, usize), // Indication that a macro has been created
}

pub fn run(
    receiver: Receiver<ZpMsg>,
    cli_sender: Sender<CliMsg>,
    mouse_sender: Sender<MouserMsg>,
    macro_sender: Sender<MacroMsg>,
) {
    let mut echo_mode = true;
    let mut layer = 0;
    let mut write_layer = 0;
    let mut keymaps: HashMap<u16, Binding> = HashMap::new();
    let mut values: HashMap<u8, f64> = HashMap::new();
    let mut released_layers = Vec::with_capacity(8);

    while let Ok(msg) = receiver.recv() {
        use ZpMsg::*;
        match msg {
            Output(event) => {
                send(&event);
            },
            Input(event) => {
                if echo_mode {
                    println!("{}", event);
                }
                let (id, value) = event.to_id();
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
                            send_to_mouse(&mouse_sender, MouserMsg::MouseX(v * value));
                        },
                        Some(Mapping::MouseY(v)) => {
                            send_to_mouse(&mouse_sender, MouserMsg::MouseY(v * value));
                        },
                        Some(Mapping::FlickX(v)) => {
                            send_to_mouse(&mouse_sender, MouserMsg::FlickX(v * value));
                        },
                        Some(Mapping::FlickY(v)) => {
                            send_to_mouse(&mouse_sender, MouserMsg::FlickY(v * value));
                        },
                        Some(Mapping::Emit(ev)) => {
                            send(&ev);
                        },
                        Some(Mapping::Trigger(idx)) =>  {
                            send_to_macro(&macro_sender, MacroMsg::Trigger(idx, value));
                        }
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
                let binding = Binding::new(mapping);
                keymaps.insert(idx, binding);
            },
            SetDeadzoneOn(button, v) => {
                let idx = button as u16 + write_layer as u16 * 256;
                if let Some(binding) = keymaps.get_mut(&idx) {
                    binding.deadzone_on = Some(v / 100.0);
                } else {
                    println!("No binding found for {} ({})", button, idx);
                }
            },
            SetDeadzoneOff(button, v) => {
                let idx = button as u16 + write_layer as u16 * 256;
                if let Some(binding) = keymaps.get_mut(&idx) {
                    binding.deadzone_off = Some(v / 100.0);
                } else {
                    println!("No binding found for {} ({})", button, idx);
                }
            },

            SetWriteLayer(v) => { write_layer = v; },
            SetFps(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFps(v));
                send_to_macro(&macro_sender, MacroMsg::SetFps(v));
            },
            SetFlickFactor(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFlickFactor(v));
            },
            SetFlickTime(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFlickTime(v));
            },
            SetFlickDeadzone(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFlickDeadzone(v));
            },
            GetFlickCalibration(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::GetFlickCalibration(v));
            },
            SetEcho(on) => {
                echo_mode = on;
            },
            CreateMacro(button, macro_type) => {
                let button = button as u16 + write_layer as u16 * 256;
                send_to_macro(&macro_sender, MacroMsg::Create(button, macro_type));
            },
            AddToMacro(mapping) => {
                send_to_macro(&macro_sender, MacroMsg::Add(mapping));
            },
            MacroCreated(button, idx) => {
                let binding = Binding::new(Mapping::Trigger(idx));
                keymaps.insert(button, binding);
            },
        }

        if !released_layers.is_empty() {
            for l in &released_layers {
                let range = (LAYER_SIZE * *l as u16)..(LAYER_SIZE * (*l as u16 + 1));
                for (k, binding) in &mut keymaps {
                    if !range.contains(k) { continue; }
                    let idx = *k as u8;
                    let prev = values.entry(idx).or_default();
                    if *prev == 0.0 { continue; }
                    let release = binding.get_mapping(0.0, *prev);
                    match release {
                        Some(Mapping::MouseX(_)) => {
                            send_to_mouse(&mouse_sender, MouserMsg::MouseX(0.0));
                        },
                        Some(Mapping::MouseY(_)) => {
                            send_to_mouse(&mouse_sender, MouserMsg::MouseY(0.0));
                        },
                        Some(Mapping::Trigger(idx)) =>  {
                            send_to_macro(&macro_sender, MacroMsg::Trigger(idx, 0.0));
                        }
                        Some(Mapping::Emit(ev)) => {
                            send(&ev);
                        },
                        _ => {},
                    };
                }
            }
            released_layers.clear();
        }
    }
}
