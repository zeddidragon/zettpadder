use pasts::{Loop};
use std::task::Poll::{self, Ready, Pending};
use stick::{Controller, Event, Listener};
use crossbeam_channel::{Sender};

pub type Message = (u8, f64);
pub struct ControllerPoller {
    listener: Listener,
    controllers: Vec<Controller>,
    sender: Sender<Message>,
    print_mode: bool,
}

impl ControllerPoller {
    pub fn new(sender: Sender<Message>, print_mode: bool) -> Self {
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            sender: sender,
            print_mode: print_mode,
        }
    }

    fn connect(&mut self, controller: Controller) -> Poll<usize> {
        if self.controllers.len() > 0 {
        } else if self.print_mode {
            println!("Input anything to see your device's input names!");
        } else {
            println!("Input anything to use this device!");
        }
        println!(
            "Connected p{}, id: {:016X}, name: {}",
            self.controllers.len() + 1,
            controller.id(),
            controller.name(),
        );
        self.controllers.push(controller);
        Pending
    }

    fn preamble(&mut self, id: usize, event: Event) -> Poll<usize> {
        if self.print_mode {
            println!("{:?}", event);
            Pending
        } else if event.to_id().1 > 0.3 {
            println!("Selected controller: p{}, {:?}", id + 1, self.controllers[id].name());
            Ready(id)
        } else {
            Pending
        }
    }

    fn relay(&mut self, event: Event) -> Poll<usize> {
        match self.sender.send(event.to_id()) {
            Err(err) => {
                println!("Unable to relay event: {:?}\n{:?}", event, err);
            },
            _ => {},
        };
        Pending
    }

    pub async fn run(&mut self) {
        let id = Loop::new(self)
            .when(|s| &mut s.listener, Self::connect)
            .poll(|s| &mut s.controllers, Self::preamble)
            .await;

        if self.print_mode { return; }

        Loop::new(self)
            .when(|s| &mut s.controllers[id], Self::relay)
            .await;
    }
}
