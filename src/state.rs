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
    rumble: (f32, f32),
    keymaps: BTreeMap<u8, Mapping>,
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
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            rumble: (0.0, 0.0),
            keymaps: keymaps,
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

    fn event(&mut self, id: usize, event: Event) -> Poll<Exit> {
        use rdev::EventType;
        let player = id + 1;
        let (button_id, button_value) = event.to_id();
        match self.keymaps.get(&button_id) {
            Some(Mapping::KeyPress(key)) => {
                if button_value > 0.6 {
                    println!("Pressing: {:?}", key);
                    send(&EventType::KeyPress(*key))
                } else if button_value < 0.4 {
                    println!("Releasing: {:?}", key);
                    send(&EventType::KeyRelease(*key))
                }
            },
            _ => {}
        }
        match event {
            Event::Disconnect => {
                self.controllers.swap_remove(id);
            }
            Event::MenuR(true) => return Ready(player),
            Event::ActionA(pressed) => {
                self.controllers[id].rumble(f32::from(u8::from(pressed)));
            }
            Event::ActionB(pressed) => {
                self.controllers[id].rumble(0.5 * f32::from(u8::from(pressed)));
            }
            Event::BumperL(pressed) => {
                self.rumble.0 = f32::from(u8::from(pressed));
                self.controllers[id].rumble(self.rumble);
            }
            Event::BumperR(pressed) => {
                self.rumble.1 = f32::from(u8::from(pressed));
                self.controllers[id].rumble(self.rumble);
            }
            _ => {}
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
