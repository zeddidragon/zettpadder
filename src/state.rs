use pasts::Loop;
use std::task::Poll::{self, Pending};
use stick::{Controller, Event, Listener};
use std::collections::{BTreeMap};
use crate::mapping::{Mapping, Binding};
use crossbeam_channel::{Sender};

type Exit = usize;

pub type EmitMsg = (Mapping, f64);

pub struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    keymaps: BTreeMap<u16, Binding>,
    states: BTreeMap<u8, f64>,
    layer: u16,
    pub emitter: Sender<EmitMsg>,
    print_mode: bool,
}

impl State {
    pub fn new(
        emitter: Sender<EmitMsg>,
        keymaps: BTreeMap<u16, Binding>,
    ) -> Self {
        let mut states = BTreeMap::new();
        let print_mode = (&keymaps).len() == 0;
        for k in keymaps.keys() {
            states.insert(*k as u8, 0.0);
        }
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            keymaps: keymaps,
            states: states,
            layer: 0,
            emitter: emitter,
            print_mode: print_mode,
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

    fn event(&mut self, _id: usize, event: Event) -> Poll<Exit> {
        if self.print_mode {
            println!("{:?}", event);
            return Pending
        }

        let (event_id, value) = event.to_id();
        let idx = event_id as u16;

        let shifted = idx + 256 * self.layer;
        let binding =
            if let Some(m) = self.keymaps.get(&shifted) { Some(m) }
            else if let Some(m) = self.keymaps.get(&idx) { Some(m) }
            else { None };
        let mapping = if let Some(binding) = binding {
            let prev = self.states[&event_id];
            self.states.insert(event_id, value);
            binding.get_mapping(value, prev)
        } else {
            None
        };

        match mapping {
            Some(Mapping::Layer(l)) => {
                self.layer = l;
            },
            Some(mapping) => {
                match self.emitter.send((mapping, value)) {
                    Err(err) => {
                        println!("Failed to emit: {:?}", err);
                    },
                    _ => {},
                }
            },
            _ => {},
        };
        Pending
    }

    pub async fn run(&mut self) {
        Loop::new(self)
            .when(|s| &mut s.listener, State::connect)
            .poll(|s| &mut s.controllers, State::event)
            .await;
    }
}
