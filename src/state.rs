use rdev;
use pasts::Loop;
use std::task::Poll::{self, Pending, Ready};
use stick::{Controller, Event, Listener};
use std::collections::{BTreeMap};
use super::mapping::Mapping;

type Exit = usize;

pub struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    keymaps: BTreeMap<u8, Mapping>,
    states: BTreeMap<u8, f64>,
    layer: u8,
}

fn send(event_type: &rdev::EventType) {
    match rdev::simulate(event_type) {
        Ok(()) => (),
        Err(rdev::SimulateError) => {
            println!("Unable to can {:?}", event_type);
        }
    }
}


impl State {
    pub fn new(keymaps : BTreeMap<u8, Mapping>) -> Self {
        let mut states = BTreeMap::new();
        for k in keymaps.keys() {
            states.insert(*k, 0.0);
        }
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            keymaps: keymaps,
            states: states,
            layer: 0,
        }
    }

    fn connect(&mut self, controller: Controller) -> Poll<Exit> {
        println!(
            "Connected p{}, id: {:016X}, name: {}",
            self.controllers.len() + 1,
            controller.id(),
            controller.name(),
        );
        self.controllers.push(controller);
        Pending
    }

    fn trigger(&mut self, mapping: Mapping, value: f64, prev: f64) {
        use Mapping::{Emit, NegPos};
        let on = 0.125;
        let off = 0.100;
        match mapping {
            Emit(event) => {
                if prev < on && value >= on {
                    println!("Pressing: {:?}", event);
                    send(&event)
                } else if prev > off && value <= off {
                    println!("Releasing: {:?}", event);
                    match event {
                        rdev::EventType::KeyPress(key) => {
                            send(&rdev::EventType::KeyRelease(key))
                        },
                        rdev::EventType::ButtonPress(btn) => {
                            send(&rdev::EventType::ButtonRelease(btn))
                        },
                        _ => {
                            println!("Don't know how to release: {:?}", event);
                        },
                    }
                }
            },
            NegPos(neg, pos) => {
                if value < 0.0 || prev < 0.0 {
                    self.trigger(Mapping::Emit(neg), -value, -prev)
                }
                if value > 0.0 || prev > 0.0 {
                    self.trigger(Mapping::Emit(pos), value, prev)
                }
            },
            _ => {}
        }
    }

    fn event(&mut self, _id: usize, event: Event) -> Poll<Exit> {
        let (event_id, value) = event.to_id();
        if let Some(mapping) = self.keymaps.get(&event_id) {
            let prev = self.states[&event_id];
            self.states.insert(event_id, value);
            self.trigger(*mapping, value, prev);
        }
        Pending
    }

    pub async fn run(&mut self) {
        Loop::new(self)
            .when(|s| &mut s.listener, State::connect)
            .poll(|s| &mut s.controllers, State::event)
            .await;
    }
}
