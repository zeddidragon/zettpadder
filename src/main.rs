// use rdev::xtask::name_to_hex;
use std::thread;
use toml::{Value};
use toml::Value::Table;
use std::env;
use std::collections::{BTreeMap};
use std::fs::{read_to_string};
use crossbeam_channel::{bounded};

mod coords;
mod function;
mod mapping;
mod controller_poller;
mod zettpadder;
use controller_poller::{ControllerPoller};
use zettpadder::{Zettpadder};

async fn event_loop() {
    let args: Vec<String> = env::args().collect();
    let mut keymaps = BTreeMap::new();
    let mut functions = Vec::new();

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        let contents = read_to_string(arg).unwrap();
        let config = match contents.parse::<Value>() {
            Ok(Table(x)) => x,
            Err(err) => {
                println!("Error parsing toml: {:?}", err);
                continue;
            },
            _ => { continue; },
        };
        if let Some(Value::String(game)) = config.get("game") {
            println!("Game: {}", game);
        } else {
            println!("No game specified");
        }
        for (key, value) in config { match (key.as_str(), value) {
            ("game", _) => {},
            ("mapping", Table(mappings)) => {
                mapping::parse_mappings(
                    0,
                    Table(mappings),
                    &mut keymaps,
                    &mut functions,
                );
            },
            ("layers", Table(layermaps)) => {
                mapping::parse_layers(
                    Table(layermaps),
                    &mut keymaps,
                    &mut functions,
                )
            },
            (key, value) => {
                println!("Unrecognized property or invalid value: {}\n{:?}", key, value)
            },
        } }
    }

    let (tx, rx) = bounded(128);
    let print_mode = (&keymaps).len() == 0;
    thread::spawn(move || {
        Zettpadder::new(rx, keymaps, functions).run();
    });
    ControllerPoller::new(tx, print_mode).run().await;
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
