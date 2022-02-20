// use rdev::xtask::name_to_hex;
use std::thread;
use std::env;
use std::path::Path;
use std::ffi::OsStr;
use crossbeam_channel::{bounded};

mod coords;
mod mapping;
mod controller_poller;
mod smoothing;
mod mouser;
mod zettpadder;
mod parsers;
mod cli;
use controller_poller::{ControllerPoller};

async fn event_loop() {
    let args: Vec<String> = env::args().collect();
    let (sender, receiver) = bounded(128);
    let (mouse_sender, mouse_receiver) = bounded(128);
    thread::spawn(move || {
        zettpadder::run(receiver, mouse_sender);
    });
    let mouse_to_main = sender.clone();
    thread::spawn(move || {
        mouser::run(mouse_to_main, mouse_receiver);
    });

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        let extension = Path::new(arg)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap();
        match extension {
            "zett" => { parsers::zett::parse(&sender, arg); },
            _ => {
                println!("Unrecognized filetype: {} ({})", arg, extension);
            },
        }
    }

    let cli_to_main = sender.clone();
    thread::spawn(move || {
        cli::run(cli_to_main);
    });
    ControllerPoller::new(sender).run().await;
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
