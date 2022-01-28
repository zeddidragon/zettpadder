use rdev;
use async_ctrlc::{CtrlC};
use toml::{Value};
use toml::Value::Table;
use futures::{
    future::{FutureExt},
    pin_mut,
    select,
};
use std::env;
// use std::collections::{HashMap, BTreeMap};
use std::fs::{read_to_string};
// use std::io::Write;

mod state;

/*
fn send(event_type: &rdev::EventType) {

    match rdev::simulate(event_type) {
        Ok(()) => (),
        Err(rdev::SimulateError) => {
            println!("Unable to can {:?}", event_type);
        }
    }
}
*/

async fn event_loop() {
    let args: Vec<String> = env::args().collect();

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        let contents = read_to_string(arg).unwrap();
        let config = contents.parse::<Value>().unwrap();
        if let Some(Table(alldefs)) = config.get("Definitions") {
            for (pad_id, defs) in alldefs {
                println!("Definitions for pad: {}", pad_id);
                if let Table(defs) = defs {
                    for (code, def) in defs {
                        println!("Read ({}): {:?}", code, def);
                    }
                }
            }
        }
    }

    // Iterate over all connected gamepads
    println!("No configuration loaded, starting monitoring mode.");

    let controller_loop = state::run().fuse();
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
