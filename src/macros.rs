use std::time::Duration;
use crossbeam_channel::{tick, Sender, Receiver};
use crate::zettpadder::{ZpMsg};
use crate::mapping::{Mapping};

const FPS: u64 = 60;  // Default loop rate

#[derive(Debug, Copy, Clone)]
enum MacroState {
    Inert, // Not running
    Starting(usize), // About to start from index specified
    Active, // Currently running
    Ending(usize), // About to end, cycled form index specified
}

#[derive(Debug, Clone)]
pub struct Macro {
    value: f64,
    is_turbo: bool,
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
    Create(u16), // Create Macro, button for being passed back
    Add(Mapping), // Add mapping to macro being constructed
    Trigger(usize, f64), // Trigger macro with supplied ID
}

fn release(sender: &Sender<ZpMsg>, mappings: &[Mapping]) {
    for m in mappings {
        match m.released() {
            Some(Mapping::Emit(ev)) => {
                send(&sender, ZpMsg::Output(ev));
            }, 
            _ => {},
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
                Create(reference) => {
                    let idx = macros.len();
                    macros.push(Macro {
                        value: 0.0,
                        is_turbo: false,
                        state: MacroState::Inert,
                        mappings: Vec::new(),
                    });
                    send(&sender, ZpMsg::MacroCreated(reference, idx));
                },
                Add(mapping) => {
                    let idx = macros.len() - 1;
                    match mapping {
                        Mapping::Emit(_) => {
                            macros[idx].mappings.push(mapping);
                        },
                        Mapping::Delay => {
                            macros[idx].mappings.push(mapping);
                        },
                        Mapping::Turbo => {
                            macros[idx].is_turbo = true;
                        },
                        _ => {},
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
                MacroState::Active => {
                    if mc.value == 0.0 || mc.is_turbo {
                        mc.state = MacroState::Ending(0);
                    }
                },
                _ => {},
            }

            match mc.state {
                MacroState::Starting(idx) => {
                    for (i, m) in mc.mappings.iter().enumerate().skip(idx) {
                        match m {
                            Mapping::Emit(ev) => {
                                send(&sender, ZpMsg::Output(*ev));
                            }, 
                            Mapping::Delay => {
                                // release(&sender, &mc.mappings[idx..i]);
                                mc.state = MacroState::Starting(i + 1);
                                continue 'macros;
                            },
                            _ => {},
                        }
                    }

                    mc.state = MacroState::Active;
                },
                MacroState::Ending(idx) => {
                    release(&sender, &mc.mappings[0..]);
                    mc.state = MacroState::Inert;
                },
                _ => {},
            }
        }
    }
}
