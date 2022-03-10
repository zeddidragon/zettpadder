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
mod macros;
mod cli;
mod overlay;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (sender, receiver) = bounded(128);
    let (mouse_sender, mouse_receiver) = bounded(24);
    let (macro_sender, macro_receiver) = bounded(24);
    let (overlay_sender, overlay_receiver) = bounded(24);
    thread::spawn(move || {
        zettpadder::run(
            receiver,
            mouse_sender,
            macro_sender,
            overlay_sender,
        );
        println!("zettpadder ded");
    });
    let mouse_to_main = sender.clone();
    thread::spawn(move || {
        mouser::run(mouse_to_main, mouse_receiver);
        println!("mouser ded");
    });
    let macro_to_main = sender.clone();
    thread::spawn(move || {
        macros::run(macro_to_main, macro_receiver);
        println!("macros ded");
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
    let controller_to_main = sender.clone();
    thread::spawn(move || {
        controller_poller::run(controller_to_main);
        println!("controller poller ded");
    });

    overlay::run(overlay_receiver); // Has to run on main thread
}
