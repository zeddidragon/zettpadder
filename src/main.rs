// use rdev::xtask::name_to_hex;
use std::thread;
use toml::{Value};
use toml::Value::Table;
use std::env;
use std::collections::{BTreeMap};
use std::fs::{read_to_string};
use crossbeam_channel::{bounded};

mod mapping;
mod state;
mod emitter;
use state::{State};
use emitter::{Emitter};

async fn event_loop() {
    let args: Vec<String> = env::args().collect();
    let mut keymaps = BTreeMap::new();

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        let contents = read_to_string(arg).unwrap();
        let config = match contents.parse::<Value>() {
            Ok(Table(x)) => x,
            _ => continue,
        };
        if let Some(Value::String(game)) = config.get("game") {
            println!("Game: {}", game);
        } else {
            println!("No game specified");
        }
        for (key, value) in config { match (key.as_str(), value) {
            ("game", _) => {},
            ("mapping", Table(mappings)) => {
                mapping::parse_mappings(&mut keymaps, Table(mappings), 0);
            },
            ("layers", Table(layermaps)) => {
                mapping::parse_layers(
                    &mut keymaps,
                    Table(layermaps));
            },
            (key, value) => {
                println!("Unrecognized property or invalid value: {}\n{:?}", key, value)
            },
        } }
    }

    let (tx, rx) = bounded(128);
    let mut zettpadder = State::new(tx, keymaps);
    thread::spawn(move || {
        Emitter::new(rx).run();
    });
    zettpadder.run().await;
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
