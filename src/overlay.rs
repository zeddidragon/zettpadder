use crossbeam_channel::{Receiver};

#[derive(Debug, Copy, Clone)]
pub enum OverlayMsg {
    Spawn,
    Exit,
}

pub fn run(receiver: Receiver<OverlayMsg>) {
    use OverlayMsg::*;
    while let Ok(msg) = receiver.recv() {
        match msg {
            Spawn => { break; },
            Exit => { return; }
        }
    }
}
