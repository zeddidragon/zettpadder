use std::fs::File;
use std::io::{self, BufRead, Error};
use std::slice::Iter;
use crossbeam_channel::{Sender};
use crate::function::{Function};
use crate::mapping::{Mapping};
use crate::zettpadder::{ZpMsg};
use super::inputs::{parse_input};
use super::outputs::{parse_output};

fn send(sender: &Sender<ZpMsg>, msg: ZpMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to relay event: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

pub fn parse(
    sender: &Sender<ZpMsg>,
    filename: &String,
) {
    let file = File::open(filename).unwrap();
    
    for line in io::BufReader::new(file).lines() {
        parse_line(sender, line);
    }
}

fn parse_outputs(iter: &mut Iter<&str>, mappings: &mut Vec<Mapping>) {
    use Mapping::{Emit, Layer, Noop};
    use rdev::EventType::{KeyPress};
    loop {
        let next = iter.next();
        if next.is_none() { return; }
        let next = next.unwrap();
        match next.to_lowercase().as_str() {
            "layer" => {
                let arg1 = iter.next().map(|v| v.parse::<u8>());
                if let Some(Ok(layer)) = arg1 {
                    mappings.push(Layer(layer));
                } else {
                    println!("Urecognized layer: {:?}", arg1);
                    mappings.push(Noop);
                }
            },
            "wasd" => {
                mappings.push(Emit(KeyPress(rdev::Key::KeyA)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyD)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyW)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyS)));
            },
            "ijkl" => {
                mappings.push(Emit(KeyPress(rdev::Key::KeyH)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyL)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyI)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyK)));
            },
            "arrows" => {
                mappings.push(Emit(KeyPress(rdev::Key::LeftArrow)));
                mappings.push(Emit(KeyPress(rdev::Key::RightArrow)));
                mappings.push(Emit(KeyPress(rdev::Key::UpArrow)));
                mappings.push(Emit(KeyPress(rdev::Key::DownArrow)));
            },
            "mouse" => {
                mappings.push(Mapping::MouseX(1.0));
                mappings.push(Mapping::MouseY(1.0));
            },
            "flick" => {
                mappings.push(Mapping::FlickX);
                mappings.push(Mapping::FlickY);
            },
            _ => {
                mappings.push(parse_output(next));
            }
        }
    }
}

fn parse_coords(
    sender: &Sender<ZpMsg>,
    iter: &mut Iter<&str>,
    x: u8, y: u8,
) {
    use Mapping::{Emit};
    let mut mappings = Vec::new();
    parse_outputs(iter, &mut mappings);
    match mappings.len() {
        0 => {
            send(sender, ZpMsg::Bind(x, Mapping::Noop));
            send(sender, ZpMsg::Bind(y, Mapping::Noop));
        },
        1 => {
            let m1 = mappings.get(0).unwrap();
            send(sender, ZpMsg::Bind(x, *m1));
            send(sender, ZpMsg::Bind(y, *m1));
        },
        2 => {
            let m1 = mappings.get(0).unwrap();
            let m2 = mappings.get(1).unwrap();
            send(sender, ZpMsg::Bind(x, *m1));
            send(sender, ZpMsg::Bind(y, *m2));
        },
        4 => {
            let m1 = mappings.get(0).unwrap();
            let m2 = mappings.get(1).unwrap();
            let m3 = mappings.get(2).unwrap();
            let m4 = mappings.get(3).unwrap();
            match (m1, m2) {
                (Emit(e1), Emit(e2)) => {
                    send(sender, ZpMsg::Bind(x, Mapping::NegPos(*e1, *e2)));
                },
                (n, p) => {
                    println!("Unable to interpret, ({:?}, {:?}", n, p);
                },
            }
            match (m3, m4) {
                (Emit(e3), Emit(e4)) => {
                    send(sender, ZpMsg::Bind(y, Mapping::NegPos(*e3, *e4)));
                },
                (n, p) => {
                    println!("Unable to interpret, ({:?}, {:?}", n, p);
                },
            }
        },
        _ => {
            println!("Unexpected amount of bindings. Expecte 0, 1, 2, or 4.");
            println!("{:?}", mappings);
        },
    }
}

fn parse_quadrant(
    sender: &Sender<ZpMsg>,
    iter: &mut Iter<&str>,
    xn: u8, xp: u8,
    yn: u8, yp: u8,
) {
    let mut mappings = Vec::new();
    parse_outputs(iter, &mut mappings);
    match mappings.len() {
        0 => {
            send(sender, ZpMsg::Bind(xn, Mapping::Noop));
            send(sender, ZpMsg::Bind(xp, Mapping::Noop));
            send(sender, ZpMsg::Bind(yn, Mapping::Noop));
            send(sender, ZpMsg::Bind(yp, Mapping::Noop));
        },
        1 => {
            let m1 = mappings.get(0).unwrap();
            send(sender, ZpMsg::Bind(xn, *m1));
            send(sender, ZpMsg::Bind(xp, *m1));
            send(sender, ZpMsg::Bind(yn, *m1));
            send(sender, ZpMsg::Bind(yp, *m1));
        },
        2 => {
            let m1 = mappings.get(0).unwrap();
            let m2 = mappings.get(1).unwrap();
            // TODO: negative effect for left/up axis
            send(sender, ZpMsg::Bind(xn, *m1));
            send(sender, ZpMsg::Bind(xp, *m1));
            send(sender, ZpMsg::Bind(yn, *m2));
            send(sender, ZpMsg::Bind(yp, *m2));
        },
        4 => {
            let m1 = mappings.get(0).unwrap();
            let m2 = mappings.get(1).unwrap();
            let m3 = mappings.get(2).unwrap();
            let m4 = mappings.get(3).unwrap();
            send(sender, ZpMsg::Bind(xn, *m1));
            send(sender, ZpMsg::Bind(xp, *m2));
            send(sender, ZpMsg::Bind(yn, *m3));
            send(sender, ZpMsg::Bind(yp, *m4));
        },
        _ => {
            println!("Unexpected amount of bindings. Expecte 0, 1, 2, or 4.");
            println!("{:?}", mappings);
        },
    }
}


fn parse_line(sender: &Sender<ZpMsg>, line: Result<String, Error>) {
    if let Err(err) = line {
        println!("Error parsing zett: {:?}", err);
        return;
    }
    let line = line.unwrap();
    let tokens = line
        .split('#') // Remove comments
        .nth(0)
        .unwrap()
        .trim_start() // Remove any indentation
        .split_whitespace()
        .collect::<Vec<_>>();
    let mut iter = tokens.iter();
    let cmd = iter.next();
    if cmd.is_none() { return;}
    let cmd = cmd.unwrap();
    match cmd.to_lowercase().as_str() {
        "fps" => {
            let arg1 = iter.next().map(|v| v.parse::<u64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetFps(v));
            } else {
                println!("Usage: fps <n>");
            }
        },
        "flickfactor" => {
            let arg1 = iter.next().map(|v| v.parse::<f64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetFlickFactor(v));
            } else {
                println!("Usage: flickfactor <n>");
            }
        },
        "flicktime" => {
            let arg1 = iter.next().map(|v| v.parse::<u64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetFlickTime(v));
            } else {
                println!("Usage: flicktime <n>");
            }
        },
        "flickdeadzone" => {
            let arg1 = iter.next().map(|v| v.parse::<f64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetFlickDeadzone(v));
            } else {
                println!("Usage: flickfactor <n>");
            }
        },
        "layer" => {
            let arg1 = iter.next().map(|v| v.parse::<u8>());
            match arg1 {
                Some(Ok(layer)) => {
                    send(sender, ZpMsg::SetWriteLayer(layer));
                },
                _ => {
                    println!("Usage: layer <n>");
                },
            }
        },
        "dpadxy" => {
            parse_quadrant(sender, &mut iter, 0x12, 0x13, 0x10, 0x11);
        },
        "povxy" => {
            parse_quadrant(sender, &mut iter, 0x1E, 0x1F, 0x1C, 0x1D);
        },
        "hatxy" => {
            parse_quadrant(sender, &mut iter, 0x16, 0x17, 0x14, 0x15);
        },
        "joyxy" => {
            parse_coords(sender, &mut iter, 0x20, 0x21);
        },
        "camxy" => {
            parse_coords(sender, &mut iter, 0x23, 0x24);
        },
        "mousexy" => {
            parse_coords(sender, &mut iter, 0x4E, 0x4F);
        },
        "actionwheelxy" => {
            parse_coords(sender, &mut iter, 0x5E, 0x5F);
        },
        _ => {
            let input = parse_input(&cmd.to_string());
            if !input.is_ok() {
                println!("Unknown command: {}", cmd);
                return;
            };
            let input = input.unwrap() as u8;
            let mut mappings = Vec::new();
            parse_outputs(&mut iter, &mut mappings);
            let parsed =
                if mappings.len() == 2 {
                    let m1 = mappings.get(0).unwrap();
                    let m2 = mappings.get(1).unwrap();
                    match (m1, m2) {
                        (Mapping::Emit(e1), Mapping::Emit(e2)) => {
                            Mapping::NegPos(*e1, *e2)
                        },
                        (_, p) => {
                            *p
                        },
                    }
                } else {
                    *mappings
                        .get(0)
                        .unwrap_or(&Mapping::Noop)
                };
            send(sender, ZpMsg::Bind(input, parsed));
        }
    }
}
