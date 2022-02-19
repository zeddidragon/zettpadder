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
mod cli;
use controller_poller::{ControllerPoller};
use zettpadder::{ZpMsg};

async fn event_loop() {
    let args: Vec<String> = env::args().collect();
    let (sender, receiver) = bounded(128);
    thread::spawn(move || {
        zettpadder::run(receiver);
    });

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        sender.send(ZpMsg::SetEcho(false));
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

    let cli_sender = sender.clone();
    thread::spawn(move || {
        cli::run(cli_sender);
    });
    ControllerPoller::new(sender).run().await;
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
