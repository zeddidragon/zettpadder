use rdev;
use pasts::Loop;
use std::task::Poll::{self, Pending};
use stick::{Controller, Event, Listener};
use std::collections::{BTreeMap};
use crate::mapping::{Mapping, Binding};
use crossbeam_channel::{Sender};

type Exit = usize;

#[derive(Debug, Copy, Clone)]
pub enum MouseMsgType {
    MoveX,
    MoveY,
    FlickX,
    FlickY,
}

pub type MouseMsg = (MouseMsgType, f64);

pub struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    keymaps: BTreeMap<u16, Binding>,
    states: BTreeMap<u8, f64>,
    layer: u16,
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
        keymaps: BTreeMap<u16, Binding>,
    ) -> Self {
        let mut states = BTreeMap::new();
        for k in keymaps.keys() {
            states.insert(*k as u8, 0.0);
        }
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            keymaps: keymaps,
            states: states,
            layer: 0,
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

    fn trigger(&self, binding: Binding, value: f64, prev: f64) -> Option<u16> {
        use Mapping::{Emit, NegPos, Mouse, Layer};
        use rdev::EventType::{
            KeyPress,
            KeyRelease,
            ButtonPress,
            ButtonRelease,
            Wheel };
        let on = binding.deadzone_on;
        let off = binding.deadzone_off;
        match binding.mapping {
            Mouse(msg_type, v) => {
                match self.mouse.send((msg_type, v * value)) {
                    Err(err) => {
                        println!("Mouse event error: ({:?}) ({:?},{})", err, msg_type, v)
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
                        _ => {
                            println!("Don't know how to release: {:?}", event);
                        },
                    }
                }
            },
            Layer(l) => {
                if prev < on && value >= on {
                    return Some(l);
                } else if prev > off && value <= off {
                    return Some(0);
                }
            },
            NegPos(neg, pos) => {
                let (mapping, value, prev) =
                    if value < 0.0 || prev < 0.0 {
                        (Some(Mapping::Emit(neg)), -value, -prev)
                    } else if value > 0.0 || prev > 0.0 {
                        (Some(Mapping::Emit(pos)), value, prev)
                    } else {
                        (None, 0.0, 0.0)
                    };
                if let Some(mapping) = mapping {
                    self.trigger(Binding {
                        mapping: mapping,
                        deadzone_on: on,
                        deadzone_off: off,
                    }, value, prev);
                }
            },
            _ => {}
        };
        None
    }

    fn event(&mut self, _id: usize, event: Event) -> Poll<Exit> {
        let (event_id, value) = event.to_id();
        let idx = event_id as u16;

        let shifted = idx + 256 * self.layer;
        let mapping =
            if let Some(m) = self.keymaps.get(&shifted) { Some(m) }
            else if let Some(m) = self.keymaps.get(&idx) { Some(m) }
            else { None };

        if let Some(mapping) = mapping {
            let prev = self.states[&event_id];
            self.states.insert(event_id, value);
            if let Some(layer) = self.trigger(*mapping, value, prev) {
                self.layer = layer;
            }
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
