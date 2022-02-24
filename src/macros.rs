use std::time::Duration;
use crossbeam_channel::{tick, Sender, Receiver};
use crate::zettpadder::{ZpMsg};
use crate::mapping::{Mapping};

const FPS: u64 = 60;  // Default loop rate

#[derive(Debug, Copy, Clone)]
pub enum MacroType {
    Simple,
    Turbo,
}

#[derive(Debug, Copy, Clone)]
enum MacroState {
    Inert, // Not running
    Starting(usize), // About to start from index specified
    Pausing(usize, usize), // Having a break after running the range specified
    Active(usize), // Currently running at index specified
    Ending(usize), // About to end, cycled form index specified
}

#[derive(Debug, Clone)]
pub struct Macro {
    value: f64,
    macro_type: MacroType,
    state: MacroState,
    mappings: Vec<Mapping>,
}

fn send(sender: &Sender<ZpMsg>, msg: ZpMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to send to zettpadder: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

#[derive(Debug, Copy, Clone)]
pub enum MacroMsg {
    SetFps(u64), // Set framerate for turbo purposes etcc
    Create(u16, MacroType), // Create Macro, button for being passed back
    Add(Mapping), // Add mapping to macro being constructed
    Trigger(usize, f64), // Trigger macro with supplied ID
}

fn release(sender: &Sender<ZpMsg>, mappings: &[Mapping]) {
    for m in mappings {
        match m.released() {
            Some(Mapping::Emit(ev)) => {
                send(&sender, ZpMsg::Output(ev));
            }, 
            Some(Mapping::Delay) => {
                break;
            }, 
            Some(Mapping::Layer(v)) => {
                send(&sender, ZpMsg::SetLayer(v));
            },
            Some(mr) => {
                println!("Unknown release: {:?}", mr);
            },
            None => {},
        }
    }
}

pub fn run(sender: Sender<ZpMsg>, receiver: Receiver<MacroMsg>) {
    let mut tick_time = Duration::from_nanos(1_000_000_000 / FPS);
    let mut ticker = tick(tick_time);
    let mut macros = Vec::new();

    loop {
        while let Ok(msg) = receiver.try_recv() {
            use MacroMsg::*;
            match msg {
                SetFps(v) => {
                    tick_time = Duration::from_nanos(1_000_000_000 / v);
                    ticker = tick(tick_time);
                },
                Create(reference, macro_type) => {
                    let idx = macros.len();
                    macros.push(Macro {
                        value: 0.0,
                        macro_type: macro_type,
                        state: MacroState::Inert,
                        mappings: Vec::new(),
                    });
                    send(&sender, ZpMsg::MacroCreated(reference, idx));
                },
                Add(mapping) => {
                    let idx = macros.len() - 1;
                    match mapping {
                        _ => {
                            macros[idx].mappings.push(mapping);
                        },
                    }
                },
                Trigger(idx, value) => {
                    macros[idx].value = value;
                },
            }
        }

        ticker.recv().unwrap();

        'macros : for mc in &mut macros {
            match mc.state {
                MacroState::Inert => {
                    if mc.value > 0.0 {
                        mc.state = MacroState::Starting(0);
                    } else {
                        continue;
                    }
                },
                MacroState::Active(idx) => {
                    if let MacroType::Turbo = mc.macro_type {
                        mc.state = MacroState::Ending(idx);
                    } else if mc.value == 0.0 {
                        mc.state = MacroState::Ending(idx);
                    }
                },
                _ => {},
            }

            match mc.state {
                MacroState::Starting(idx) => {
                    if idx > 0 {
                    }
                    for (i, m) in mc.mappings.iter().enumerate().skip(idx) {
                        match m {
                            Mapping::Emit(ev) => {
                                send(&sender, ZpMsg::Output(*ev));
                            }, 
                            Mapping::Delay => {
                                mc.state = MacroState::Pausing(idx, i);
                                continue 'macros;
                            },
                            Mapping::Layer(v) => {
                                send(&sender, ZpMsg::SetLayer(*v));
                            },
                            _ => {
                                println!("Uknown input: {:?}", m);
                            },
                        }
                    }

                    mc.state = MacroState::Active(idx);
                },
                MacroState::Pausing(from, to) => {
                    release(&sender, &mc.mappings[from..to]);
                    mc.state = MacroState::Starting(to + 1);
                },
                MacroState::Ending(idx) => {
                    release(&sender, &mc.mappings[idx..]);
                    mc.state = MacroState::Inert;
                },
                _ => {},
            }
        }
    }
}
