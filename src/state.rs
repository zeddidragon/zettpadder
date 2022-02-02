use rdev;
use pasts::Loop;
use std::task::Poll::{self, Pending};
use stick::{Controller, Event, Listener};
use std::collections::{BTreeMap};
use super::mapping::Mapping;
use crossbeam_channel::{Sender};

type Exit = usize;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MouseAxis { X = 0, Y = 1 }
pub struct MouseMsg {
    pub value: f64,
    pub axis: MouseAxis,
}

pub struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    keymaps: BTreeMap<u8, Mapping>,
    states: BTreeMap<u8, f64>,
    // layer: u8,
    pub mouse: Sender<MouseMsg>,
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
    pub fn new(
        mouse: Sender<MouseMsg>,
        keymaps: BTreeMap<u8, Mapping>,
    ) -> Self {
        let mut states = BTreeMap::new();
        for k in keymaps.keys() {
            states.insert(*k, 0.0);
        }
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            keymaps: keymaps,
            states: states,
            // layer: 0,
            mouse: mouse,
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

    fn trigger(&self, mapping: Mapping, value: f64, prev: f64) {
        use Mapping::{Emit, NegPos};
        use rdev::EventType::{
            KeyPress,
            KeyRelease,
            ButtonPress,
            ButtonRelease,
            MouseMoveRelative,
            Wheel };
        let on = 0.125;
        let off = 0.100;
        match mapping {
            Emit(MouseMoveRelative { delta_x, delta_y }) => {
                let (v, axis) = if delta_x.abs() > 0.0 {
                    (delta_x, MouseAxis::X)
                } else {
                    (delta_y, MouseAxis::Y)
                };
                let message = MouseMsg {
                    value: v * value,
                    axis: axis,
                };
                match self.mouse.send(message) {
                    Err(err) => {
                        println!("Mouse event error: ({:?}) ({},{})", err, delta_x, delta_y)
                    },
                    _ => {},
                }
            },
            Emit(event) => {
                if prev < on && value >= on {
                    send(&event)
                } else if prev > off && value <= off {
                    match event {
                        KeyPress(key) => {
                            send(&KeyRelease(key))
                        },
                        ButtonPress(btn) => {
                            send(&ButtonRelease(btn))
                        },
                        Wheel { delta_x: _x, delta_y: _y } => {
                            // No need to release wheel action
                        },
                        MouseMoveRelative { delta_x: _x, delta_y: _y } => {
                            // TODO: Stop emitting mouse motion
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
