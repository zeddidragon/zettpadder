// use rdev::xtask::name_to_hex;
use std::thread;
use std::env;
use std::path::Path;
use std::ffi::OsStr;
use crossbeam_channel::{bounded};

mod coords;
mod function;
mod mapping;
mod controller_poller;
mod smoothing;
mod zettpadder;
mod parsers;
use controller_poller::{ControllerPoller};

async fn event_loop() {
    let args: Vec<String> = env::args().collect();
    let (tx, rx) = bounded(128);
    thread::spawn(move || {
        zettpadder::run(rx);
    });

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        let extension = Path::new(arg)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap();
        match extension {
            "zett" => { parsers::zett::parse(&tx, arg); },
            _ => {
                println!("Unrecognized filetype: {} ({})", arg, extension);
            },
        }
    }

    let print_mode = args.len() < 2;
    ControllerPoller::new(tx, print_mode).run().await;
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
