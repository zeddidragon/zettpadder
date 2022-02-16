// use rdev::xtask::name_to_hex;
use std::thread;
use toml::Value::{self, Table};
use std::env;
use std::fs::{read_to_string};
use crossbeam_channel::{bounded, Sender};

mod coords;
mod function;
mod mapping;
mod controller_poller;
mod smoothing;
mod zettpadder;
use controller_poller::{ControllerPoller};
use zettpadder::{ZpMsg};

fn send(sender: &Sender<ZpMsg>, msg: ZpMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to relay event: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

async fn event_loop() {
    let args: Vec<String> = env::args().collect();
    let (tx, rx) = bounded(128);
    thread::spawn(move || {
        zettpadder::run(rx);
    });

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
            ("fps", Value::Integer(v)) => {
                send(&tx, ZpMsg::SetFps(v as u64));
            },
            ("flickFactor", Value::Integer(v)) => {
                send(&tx, ZpMsg::SetFlickFactor(v as f64));
            },
            ("flickFactor", Value::Float(v)) => {
                send(&tx, ZpMsg::SetFlickFactor(v));
            },
            ("flickTime", Value::Integer(v)) => {
                send(&tx, ZpMsg::SetFlickTime(v as u64));
            },
            ("flickDeadzone", Value::Float(v)) => {
                send(&tx, ZpMsg::SetFlickDeadzone(v as f64));
            },
            ("mapping", Table(mappings)) => {
                mapping::parse_mappings(&tx, Table(mappings));
            },
            ("layers", Table(layermaps)) => {
                mapping::parse_layers(&tx, Table(layermaps));
            },
            (key, value) => {
                println!("Unrecognized property or invalid value: {}\n{:?}", key, value)
            },
        } }
    }

    let print_mode = args.len() < 2;
    ControllerPoller::new(tx, print_mode).run().await;
}

fn main() {
    pasts::block_on(event_loop());
    println!("Bye bye!");
}
