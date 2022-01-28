use gilrs;
use rdev;
use ctrlc;
use toml::{Value};
use toml::Value::Table;
use std::env;
use std::collections::{HashMap, BTreeMap};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::sync::{Arc, atomic};
use std::{thread, time};

fn send(event_type: &rdev::EventType) {

    match rdev::simulate(event_type) {
        Ok(()) => (),
        Err(rdev::SimulateError) => {
            println!("Unable to can {:?}", event_type);
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut gilrs = gilrs::Gilrs::new().unwrap();
    let sleeptime = time::Duration::from_millis(10);

    for arg in args.iter().skip(1) {
        println!("Reading definitions from {}", arg);
        let contents = read_to_string(arg)?;
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

    // If no configuration, just emit events
    {
        let mut codes = HashMap::new();
        let should_run = Arc::new(atomic::AtomicBool::new(true));
        let should_run_borrowed = should_run.clone();
        ctrlc::set_handler(move || {
            println!("Bye bye!");
            if should_run_borrowed.load(atomic::Ordering::Relaxed) {
                should_run_borrowed.store(false, atomic::Ordering::Relaxed);
            }
        }).expect("Error setting Ctrl-C handler");

        // Iterate over all connected gamepads
        for (_id, gamepad) in gilrs.gamepads() {
            let padcodes = BTreeMap::new();
            codes.insert(_id, padcodes);
            println!("{} is {:?}", gamepad.name(), gamepad.mapping_source());
        }

        println!("No configuration loaded, starting monitoring mode.");
        while should_run.load(atomic::Ordering::Relaxed) {
            // Examine new events
            while let Some(gilrs::Event { id, event, time: _ }) = gilrs.next_event() {
                let padcodes = codes.get_mut(&id).expect("Gamepad not found!");
                match event {
                    gilrs::EventType::ButtonPressed(button, code) => {
                        println!("pressed: {:?} {:?}", button, code);
                        send(&rdev::EventType::KeyPress(rdev::Key::KeyS));
                        padcodes.insert(code.into_u32(), button);
                    },
                    gilrs::EventType::ButtonReleased(button, code) => {
                        println!("released: {:?} {:?}", button, code);
                        send(&rdev::EventType::KeyRelease(rdev::Key::KeyS));
                    },
                    _ => {
                        println!("{:?}", event);
                    },
                }
            }
            thread::sleep(sleeptime);
        }

        for(id, padcodes) in codes.into_iter() {
            let mut buffer = File::create("output.toml")?;
            writeln!(&mut buffer, "[Definitions.{}]", id)?;
            for(code, button) in padcodes {
                writeln!(&mut buffer, "{} = \"{:?}\"", code, button)?;
            }
            writeln!(&mut buffer, "")?;
        }
    }
    Ok(())
}
