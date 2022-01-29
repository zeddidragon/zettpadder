// use rdev::xtask::name_to_hex;
use async_ctrlc::{CtrlC};
use toml::{Value};
use toml::Value::Table;
use futures::{
    future::{FutureExt},
    pin_mut,
    select,
};
use std::env;
use std::collections::{HashMap, BTreeMap};
use std::fs::{read_to_string};
// use std::io::Write;

mod mapping;
mod state;

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
                mapping::parse_mappings(&mut keymaps, Table(mappings));
            },
            ("layers", _) => {}, // TODO
            (key, value) => {
                println!("Unrecognized property or invalid value: {}\n{:?}", key, value)
            },
        } }
    }
    println!("Mapped: {:?}", keymaps);

    // Iterate over all connected gamepads
    println!("No configuration loaded, starting monitoring mode.");

    let mut zettpadder = state::State::new(keymaps);
    let controller_loop = zettpadder.run().fuse();
    let ctrlc_loop = CtrlC::new().expect("cannot create Ctrl+C handler").fuse();
    pin_mut!(controller_loop, ctrlc_loop);
    select! {
        _ = controller_loop => println!("Controller quitted"),
        _ = ctrlc_loop => println!("ctrlc quitted"),
    }
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
