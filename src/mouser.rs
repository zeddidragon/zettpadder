use rdev;
use crossbeam_channel::{Receiver, tick};
use super::state::{MouseMsg, MouseAxis};
use std::time::Duration;

pub struct Mouser {
    x: f64,
    y: f64,
    pub receiver: Receiver<MouseMsg>,
}

impl Mouser {
    pub fn new(
        receiver: Receiver<MouseMsg>,
    ) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            receiver: receiver,
        }
    }

    pub fn run(&mut self) {
        use rdev::EventType::{MouseMoveRelative};
        let ticker = tick(Duration::from_nanos(1_000_000_000 / 120));
        loop {
            ticker.recv().unwrap();
            while let Ok(v) = self.receiver.try_recv() {
                if v.axis == MouseAxis::X {
                    self.x = v.value;
                } else {
                    self.y = v.value;
                }
            }
            if self.x.abs() > 0.1 || self.y.abs() > 0.1 {
                let event = MouseMoveRelative {
                    delta_x: self.x * 10.0,
                    delta_y: self.y * 10.0,
                };
                match rdev::simulate(&event) {
                    Ok(()) => (),
                    Err(rdev::SimulateError) => {
                        println!("Unable to can {:?}", event);
                    }
                }
            }
        }
    }
}
