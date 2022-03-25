use rdev;
use stick;
use std::collections::{HashMap};
use crossbeam_channel::{Sender, Receiver};
use crate::mapping::{Binding, Mapping};
use crate::ring::{Ring, RingFactory};
use crate::mouser::{MouserMsg, MousePriority};
use crate::macros::{MacroMsg, MacroType};
use crate::overlay::{OverlayMsg};

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

fn send_to_overlay(sender: &Sender<OverlayMsg>, msg: OverlayMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to send to overlay: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

#[derive(Debug, Copy, Clone)]
pub enum ZpMsg {
    Exit, // Stop the program
    Output(rdev::EventType), // Perform output directly
    Input(stick::Event), // Input from controller to process mapping for
    SetLayer(u8), // Layer used in future inputs
    SetWriteLayer(u8), // Layer used in future assignments
    SetFps(u64), // Cycle rate of main loop
    SetTapTime(u64), // Set amount of frame for a tap
    SetMouseCalibration(f64), // Mouse motion of one radian
    SetInGameMouse(f64), // In-game mouse sensitivity
    SetFlickTime(u64, bool), // Duration of a flick
    SetFlickDeadzone(f64), // Deadzone before performing a flick
    Bind(u8, Mapping), // Bind output to button
    SetDeadzoneOn(u8, f64), // Deadzone before binding enables
    SetDeadzoneOff(u8, f64), // Deadzone before binding disables
    GetFlickCalibration(f64), // Display data to help calibrate flick factor
    SetMousePriority(MousePriority), // Which type of mouse input to prioritize
    SetEcho(bool), // Echo mode, which repeats your keys back
    CreateMacro(u8, MacroType), // Request to create a macro, assign to this button
    AddToMacro(Mapping), // Add Mapping to currently constructed macro
    MacroCreated(u16, usize), // Indication that a macro has been created
    BuildRing(u8, u8), // Create a ring using inputs for x and y
    RingDone, // Finalize ring being built
    BindRingX(Mapping), // Bind action to ring X
    BindRingY(Mapping), // Bind action to ring Y
    BindRing(Mapping), // Bind action to ring length
    SetRingDeadzoneOn(f64), // Deadzone of axis in ring
    SetRingDeadzoneOff(f64), // Deadzone of axis in ring
    SetRingOn(f64), // Deadzone before ring binding enables
    SetRingOff(f64), // Deadzone before ring binding disables
    SpawnOverlay,
}

pub fn run(
    receiver: Receiver<ZpMsg>,
    mouse_sender: Sender<MouserMsg>,
    macro_sender: Sender<MacroMsg>,
    overlay_sender: Sender<OverlayMsg>,
) {
    let mut echo_mode = true;
    let mut layer: u8 = 0;
    let mut write_layer = 0;
    let mut keymaps: HashMap<u16, Binding> = HashMap::new();
    let mut values: HashMap<u8, f64> = HashMap::new();
    let mut released_layers: Vec<u8> = Vec::with_capacity(8);
    let mut rings: Vec<Ring> = Vec::new();
    let mut ring_factory = RingFactory::default();

    while let Ok(msg) = receiver.recv() {
        use ZpMsg::*;
        match msg {
            Exit => {
                send_to_overlay(&overlay_sender, OverlayMsg::Exit);
            },
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
                    handle_mapping(
                        mapping,
                        value,
                        &mut layer,
                        &mut released_layers,
                        &mut rings,
                        &mouse_sender,
                        &macro_sender,
                    );
                    *prev = value;
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

            SetLayer(v) => { layer = v; }
            SetWriteLayer(v) => { write_layer = v; },
            SetFps(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFps(v));
                send_to_macro(&macro_sender, MacroMsg::SetFps(v));
            },
            SetTapTime(v) => {
                send_to_macro(&macro_sender, MacroMsg::SetTapTime(v));
            },
            SetMouseCalibration(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetMouseCalibration(v));
            },
            SetInGameMouse(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetInGameMouse(v));
            },
            SetFlickTime(v, b) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFlickTime(v, b));
            },
            SetFlickDeadzone(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetFlickDeadzone(v));
            },
            GetFlickCalibration(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::GetFlickCalibration(v));
            },
            SetMousePriority(v) => {
                send_to_mouse(&mouse_sender, MouserMsg::SetMousePriority(v));
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
            BuildRing(x, y) => {
                let x = x as u16 + write_layer as u16 * 256;
                let y = y as u16 + write_layer as u16 * 256;

                let idx = rings.len();
                ring_factory.clear();

                let xbinding = Binding::new(Mapping::RingX(idx));
                let ybinding = Binding::new(Mapping::RingY(idx));
                keymaps.insert(x, xbinding);
                keymaps.insert(y, ybinding);
            },
            RingDone => {
                if let Ok(ring) = ring_factory.build() {
                    rings.push(ring);
                } else {
                    println!("Unable to build ring!");
                }
                ring_factory.clear();
            },
            BindRingX(mapping) => {
                ring_factory.bind_x(mapping);
            },
            BindRingY(mapping) => {
                ring_factory.bind_y(mapping);
            },
            BindRing(mapping) => {
                ring_factory.bind_r(mapping);
            },
            SetRingDeadzoneOn(v) => {
                ring_factory.with_deadzone_on(v / 100.0);
            },
            SetRingDeadzoneOff(v) => {
                ring_factory.with_deadzone_off(v / 100.0);
            },
            SetRingOn(v) => {
                ring_factory.with_ring_on(v / 100.0);
            },
            SetRingOff(v) => {
                ring_factory.with_ring_off(v / 100.0);
            },
            SpawnOverlay => {
                send_to_overlay(&overlay_sender, OverlayMsg::Spawn);
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

fn handle_mapping(
    mapping: Option<Mapping>,
    value: f64,
    layer: &mut u8,
    released_layers: &mut Vec<u8>,
    rings: &mut Vec<Ring>,
    mouse_sender: &Sender<MouserMsg>,
    macro_sender: &Sender<MacroMsg>
) {
    match mapping {
        Some(Mapping::Layer(l)) => {
            if l != *layer {
                // Untrigger any chorded keys
                if *layer > 0 {
                    released_layers.push(*layer);
                }
                *layer = l;
            }
        },
        Some(Mapping::MouseX(v)) => {
            send_to_mouse(&mouse_sender, MouserMsg::MouseX(v));
        },
        Some(Mapping::MouseY(v)) => {
            send_to_mouse(&mouse_sender, MouserMsg::MouseY(v));
        },
        Some(Mapping::FlickX(v)) => {
            send_to_mouse(&mouse_sender, MouserMsg::FlickX(v));
        },
        Some(Mapping::FlickY(v)) => {
            send_to_mouse(&mouse_sender, MouserMsg::FlickY(v));
        },
        Some(Mapping::Emit(ev)) => {
            send(&ev);
        },
        Some(Mapping::Trigger(idx)) =>  {
            send_to_macro(&macro_sender, MacroMsg::Trigger(idx, value));
        }
        Some(Mapping::RingX(idx)) => {
            let xmapping = rings
                .get_mut(idx)
                .and_then(|r| r.nudge_x(value));
            handle_mapping(
                xmapping,
                value,
                layer,
                released_layers,
                rings,
                &mouse_sender,
                &macro_sender,
            );
            let rmapping = rings
                .get_mut(idx)
                .and_then(|r| r.check_ring());
            handle_mapping(
                rmapping,
                value,
                layer,
                released_layers,
                rings,
                &mouse_sender,
                &macro_sender,
            );
        },
        Some(Mapping::RingY(idx)) => {
            let ymapping = rings
                .get_mut(idx)
                .and_then(|r| r.nudge_y(value));
            handle_mapping(
                ymapping,
                value,
                layer,
                released_layers,
                rings,
                &mouse_sender,
                &macro_sender,
            );
            let rmapping = rings
                .get_mut(idx)
                .and_then(|r| r.check_ring());
            handle_mapping(
                rmapping,
                value,
                layer,
                released_layers,
                rings,
                &mouse_sender,
                &macro_sender,
            );
        },
        Some(Mapping::Noop) => {},
        None => {},
        unx => {
            println!("Received: {:?}, which is unexpected", unx);
        }
    };
}
