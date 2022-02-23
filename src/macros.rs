use std::time::Duration;
use crossbeam_channel::{tick, Sender, Receiver};
use crate::zettpadder::{ZpMsg};
use crate::mapping::{Mapping};

const FPS: u64 = 60;  // Default loop rate

enum MacroState {
    Inert,
    Starting,
    Active,
    Ending,
}

pub struct Macro {
    value: f64,
    idx: usize,
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
    Create,
    Add(Mapping),
    Trigger(usize, f64),
}

pub fn run(sender: Sender<ZpMsg>, receiver: Receiver<MacroMsg>) {
    let mut tick_time = Duration::from_nanos(1_000_000_000 / FPS);
    let mut ticker = tick(tick_time);
    let mut macros = Vec::new();

    loop {
        while let Ok(msg) = receiver.try_recv() {
            use MacroMsg::*;
            match msg {
                Create => {
                    let idx = macros.len();
                    macros.push(Macro {
                        value: 0.0,
                        idx: 0,
                        state: MacroState::Inert,
                        mappings: Vec::new(),
                    });
                    send(&sender, ZpMsg::MacroCreated(idx));
                },
                Add(mapping) => {
                    let idx = macros.len() - 1;
                    match mapping {
                        Mapping::Emit(_) => {
                            macros[idx].mappings.push(mapping);
                        },
                        _ => {},
                    }
                },
                Trigger(idx, value) => {
                    macros[idx].value = value;
                    if value == 0.0 {
                        macros[idx].idx = 0;
                    }
                },
            }
        }

        ticker.recv().unwrap();

        for mc in macros.iter_mut() {
            match mc.state {
                MacroState::Inert => {
                    if mc.value > 0.0 {
                        mc.state = MacroState::Starting;
                    }
                },
                MacroState::Active => {
                    if mc.value == 0.0 {
                        mc.state = MacroState::Ending;
                    }
                },
                _ => {},
            }

            match mc.state {
                MacroState::Starting => {
                    for m in &mc.mappings {
                        match m {
                            Mapping::Emit(ev) => {
                                send(&sender, ZpMsg::Output(*ev));
                            }, 
                            _ => {},
                        }
                    }
                    mc.state = MacroState::Active;
                },
                MacroState::Ending => {
                    for m in &mc.mappings {
                        match m.released() {
                            Some(Mapping::Emit(ev)) => {
                                send(&sender, ZpMsg::Output(ev));
                            }, 
                            _ => {},
                        }
                    }
                    mc.state = MacroState::Inert;
                },
                _ => {},
            }
        }
    }
}
