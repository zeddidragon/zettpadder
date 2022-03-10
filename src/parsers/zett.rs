use std::fs::File;
use std::io::{self, BufRead};
use std::slice::Iter;
use std::iter::Peekable;
use crossbeam_channel::{Sender};
use crate::zettpadder::{ZpMsg};
use crate::mapping::{Mapping};
use crate::macros::{MacroType};
use super::inputs::{parse_input, ZettInput};
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
        if let Err(err) = line {
            println!("Error parsing zett: {:?}", err);
            continue;
        }
        parse_line(sender, line.unwrap());
    }
    // Reset write layer so next input can start fresh
    send(sender, ZpMsg::SetWriteLayer(0));
    // With bindings assigned, echo mode can be turned off
    send(sender, ZpMsg::SetEcho(false));
}

#[derive(Default)]
struct ZettOpts {
    deadzone_on: Option<f64>,
    deadzone_off: Option<f64>,
    macro_type: Option<MacroType>,
    flick_factor: Option<f64>,
    is_flick: bool,
}
 
fn parse_outputs(
    iter: &mut Peekable<Iter<&str>>,
    mappings: &mut Vec<Mapping>,
) -> ZettOpts {
    use Mapping::{Emit, Noop};
    use rdev::EventType::{KeyPress, Wheel};
    let mut opts = ZettOpts::default();
    loop {
        let next = iter.peek();
        if next.is_none() { break; }
        let next = next.unwrap();
        match next.to_lowercase().as_str() {
            "!" => {
                opts.macro_type = Some(MacroType::Simple);
            },
            "!!" => {
                opts.macro_type = Some(MacroType::Turbo);
            },
            "!Turbo" => {
                opts.macro_type = Some(MacroType::Turbo);
            },
            "!." => {
                opts.macro_type = Some(MacroType::HoldTap);
            },
            "!HoldTap" => {
                opts.macro_type = Some(MacroType::HoldTap);
            },
            "," => {
                mappings.push(Mapping::Delay);
            },
            "scrolly" => {
                opts.macro_type = Some(MacroType::Turbo);
                mappings.push(Emit(Wheel { delta_x: 0, delta_y: -5 }));
            },
            "layer" => {
                iter.next();
                let arg1 = iter.next().map(|v| v.parse::<u8>());
                if let Some(Ok(layer)) = arg1 {
                    mappings.push(Mapping::Layer(layer));
                } else {
                    println!("Urecognized layer: {:?}", arg1);
                    mappings.push(Noop);
                }
                continue; // Continue manually 'cause we next-ed
            },
            "wasd" => {
                mappings.push(Emit(KeyPress(rdev::Key::KeyA)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyD)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyW)));
                mappings.push(Emit(KeyPress(rdev::Key::KeyS)));
            },
            "ijkl" => {
                mappings.push(Emit(KeyPress(rdev::Key::KeyJ)));
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
                iter.next();
                let arg1 = iter
                    .peek()
                    .map(|v| v.parse::<f64>());
                let factor =
                    if let Some(Ok(v)) = arg1 {
                        iter.next();
                        v
                    } else {
                        1.0
                    };
                mappings.push(Mapping::MouseX(factor));
                mappings.push(Mapping::MouseY(factor));
                continue; // Continue manually 'cause we next-ed
            },
            "flick" => {
                iter.next();
                opts.is_flick = true;
                let arg1 = iter.peek().map(|v| v.parse::<f64>());
                if let Some(Ok(v)) = arg1 {
                    opts.flick_factor = Some(v);
                    iter.next();
                }
                mappings.push(Mapping::FlickX(1.0));
                mappings.push(Mapping::FlickY(1.0));
                continue; // Continue manually 'cause we next-ed
            },
            "deadzone" => {
                iter.next();
                let arg1 = iter.next().map(|v| v.parse::<f64>());
                let arg2 = iter.peek().map(|v| v.parse::<f64>());
                if let Some(Ok(dz)) = arg1 {
                    opts.deadzone_on = Some(dz);
                } else {
                    println!("Urecognized deadzone value: {:?}", arg1);
                }

                if let Some(Ok(dz)) = arg2 {
                    opts.deadzone_off = Some(dz);
                    iter.next();
                }
                continue; // Continue manually 'cause we next-ed
            },
            _ => {
                match parse_output(next) {
                    Noop => {
                        println!("Unable to understand mapping: {}", next);
                        return opts;
                    },
                    mapping => {
                        mappings.push(mapping);
                    },
                }
            }
        }
        iter.next();
    }
    opts
}

pub fn parse_line(
    sender: &Sender<ZpMsg>,
    line: String,
) {
    let tokens = line
        .split('#') // Remove comments
        .nth(0)
        .unwrap()
        .trim_start() // Remove any indentation
        .split_whitespace()
        .collect::<Vec<_>>();
    let mut iter = tokens.iter().peekable();
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
        "taptime" => {
            let arg1 = iter.next().map(|v| v.parse::<u64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetTapTime(v));
            } else {
                println!("Usage: taptime <n>");
            }
        },
        "mousecalibration" => {
            let arg1 = iter.next().map(|v| v.parse::<f64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetMouseCalibration(v));
            } else {
                println!("Usage: mousecalibration <n>");
            }
        },
        "ingamemouse" => {
            let arg1 = iter.next().map(|v| v.parse::<f64>());
            if let Some(Ok(v)) = arg1 {
                send(sender, ZpMsg::SetInGameMouse(v));
            } else {
                println!("Usage: ingamemouse <n>");
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
                println!("Usage flickdeadzone <n>");
            }
        },
        "mousepriority" => {
            use crate::mouser::MousePriority;
            let arg1 = iter.next().and_then(|v| {
                match *v {
                    "flick" => Some(MousePriority::Flick),
                    "motion" => Some(MousePriority::Motion),
                    "mixed" => Some(MousePriority::Mixed),
                    _ => None,
                }
            });
            if let Some(v) = arg1 {
                send(sender, ZpMsg::SetMousePriority(v));
            } else {
                println!("Usage flickdeadzone <n>");
            }
        },
        "deadzone" => {
            let arg1 = iter.next().and_then(|v| parse_input(&v.to_string()));
            let arg2 = iter.next().map(|v| v.parse::<f64>());
            let arg3 = iter.next().map(|v| v.parse::<f64>());
            match (arg1, arg2, arg3) {
                ( Some(ZettInput::Axis(axis)),
                  Some(Ok(v1)),
                  Some(Ok(v2)) ) => {
                    let dz_on = v1.max(v2);
                    let dz_off = v1.min(v2);
                    send(sender, ZpMsg::SetDeadzoneOn(axis, dz_on));
                    send(sender, ZpMsg::SetDeadzoneOff(axis, dz_off));
                },
                ( Some(ZettInput::Axis(axis)),
                  Some(Ok(dz)),
                  None ) => {
                    send(sender, ZpMsg::SetDeadzoneOn(axis, dz));
                },
                ( Some(ZettInput::Coords(ax, ay)),
                  Some(Ok(v1)),
                  Some(Ok(v2)) ) => {
                    // TODO: Other types of deadzones than cross
                    let dz_on = v1.max(v2);
                    let dz_off = v1.min(v2);
                    send(sender, ZpMsg::SetDeadzoneOn(ax, dz_on));
                    send(sender, ZpMsg::SetDeadzoneOff(ax, dz_off));
                    send(sender, ZpMsg::SetDeadzoneOn(ay, dz_on));
                    send(sender, ZpMsg::SetDeadzoneOff(ay, dz_off));
                },
                ( Some(ZettInput::Coords(ax, ay)),
                  Some(Ok(dz)),
                  None ) => {
                    // TODO: Other types of deadzones than cross
                    send(sender, ZpMsg::SetDeadzoneOn(ax, dz));
                    send(sender, ZpMsg::SetDeadzoneOn(ay, dz));
                },
                (_, _, _) => {
                    println!("Usage: deadzone <axis or coords> <on> [<off>]");
                },
            }
        },
        "calibrate" => {
            let v = iter
                .next()
                .map(|v| v.parse::<f64>().unwrap_or(1.0))
                .unwrap_or(1.0);
            send(sender, ZpMsg::GetFlickCalibration(v));
        },
        "echo" => {
            let arg1 = iter
                .next()
                .unwrap_or(&"on");
            let v = if *arg1 == "off" { false } else { true };
            send(sender, ZpMsg::SetEcho(v));
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
        "overlay" => {
            send(sender, ZpMsg::SpawnOverlay);
        },
        "exit" => {
            send(sender, ZpMsg::Exit);
        },
        _ => {
            let input = parse_input(&cmd.to_string());
            if !input.is_some() {
                println!("Unknown command: {}", cmd);
                return;
            };
            let input = input.unwrap();
            let mut mappings = Vec::new();
            let opts = parse_outputs(&mut iter, &mut mappings);

            use ZettInput::*;
            use Mapping::{Noop, Emit, NegPos};

            match (input, &mappings[..], &opts) {
                (Single(button), [], _) => {
                    send(sender, ZpMsg::Bind(button, Noop));
                },
                (Single(button), mappings, ZettOpts {
                    macro_type: Some(macro_type),
                .. }) => {
                    send(sender, ZpMsg::CreateMacro(button, *macro_type));
                    for m in mappings {
                        send(sender, ZpMsg::AddToMacro(*m));
                    }
                }
                (Single(button), [mapping], _) => {
                    send(sender, ZpMsg::Bind(button, *mapping));
                },
                (Single(_), ms, _) => {
                    println!("Too many outputs for one input! Expected 0 or 1, got {}", ms.len());
                    return;
                },
                (Axis(axis), [], _) => {
                    send(sender, ZpMsg::Bind(axis, Noop));
                },
                (Axis(axis), mappings, ZettOpts {
                    macro_type: Some(macro_type),
                .. }) => {
                    send(sender, ZpMsg::CreateMacro(axis, *macro_type));
                    for m in mappings {
                        send(sender, ZpMsg::AddToMacro(*m));
                    }
                }
                (Axis(axis), [mapping], _) => {
                    send(sender, ZpMsg::Bind(axis, *mapping));
                },
                (Axis(axis), [Emit(eneg), Emit(epos)], _) => {
                    let mapping = NegPos(*eneg, *epos);
                    send(sender, ZpMsg::Bind(axis, mapping));
                },
                (Axis(_), ms, _) => {
                    println!("Too many outputs for one axis! Expected 0, 1, or 2, got {}", ms.len());
                    return;
                },
                (Coords(ax, ay), [], _) => {
                    send(sender, ZpMsg::Bind(ax, Noop));
                    send(sender, ZpMsg::Bind(ay, Noop));
                },
                (Coords(ax, ay), [mx, my], _) => {
                    send(sender, ZpMsg::Bind(ax, *mx));
                    send(sender, ZpMsg::Bind(ay, *my));
                },
                (Coords(ax, ay), [
                        Emit(exneg), Emit(expos),
                        Emit(eyneg), Emit(eypos),
                ], _) => {
                    let mx = NegPos(*exneg, *expos);
                    let my = NegPos(*eyneg, *eypos);
                    send(sender, ZpMsg::Bind(ax, mx));
                    send(sender, ZpMsg::Bind(ay, my));
                },
                (Coords(_, _), ms, _) => {
                    println!("Incorrect amount of inputs for one axis pair! Expected 0, 2, or 4, got {}", ms.len());
                    return;
                },
                (Quartet(w, e, n, s), [], _) => {
                    send(sender, ZpMsg::Bind(w, Noop));
                    send(sender, ZpMsg::Bind(e, Noop));
                    send(sender, ZpMsg::Bind(n, Noop));
                    send(sender, ZpMsg::Bind(s, Noop));
                },
                // Todo: Quartet -> Axis
                (Quartet(w, e, n, s), [mw, me, mn, ms], _) => {
                    send(sender, ZpMsg::Bind(w, *mw));
                    send(sender, ZpMsg::Bind(e, *me));
                    send(sender, ZpMsg::Bind(n, *mn));
                    send(sender, ZpMsg::Bind(s, *ms));
                },
                (Quartet(_, _, _, _), ms, _) => {
                    println!("Incorrect amount of inputs for a button set! Expected 0 or 4, got {}", ms.len());
                    return;
                },
            }

            if opts.is_flick {
                if let ZettOpts { flick_factor: Some(v), .. } = &opts {
                    send(sender, ZpMsg::SetMouseCalibration(*v));
                }

                if let ZettOpts { deadzone_on: Some(dz), .. } = &opts {
                    send(sender, ZpMsg::SetFlickDeadzone(*dz));
                }
                return;
            }

            // Post-assignment configuration
            if let ZettOpts { deadzone_on: Some(dz), .. } = &opts {
                match input {
                    Single(trigger) => {
                        send(sender, ZpMsg::SetDeadzoneOn(trigger, *dz));
                    },
                    Axis(axis) => {
                        send(sender, ZpMsg::SetDeadzoneOn(axis, *dz));
                    },
                    Coords(ax, ay) => {
                        send(sender, ZpMsg::SetDeadzoneOn(ax, *dz));
                        send(sender, ZpMsg::SetDeadzoneOn(ay, *dz));
                    },
                    Quartet(w, e, n, s) => {
                        send(sender, ZpMsg::SetDeadzoneOn(w, *dz));
                        send(sender, ZpMsg::SetDeadzoneOn(e, *dz));
                        send(sender, ZpMsg::SetDeadzoneOn(n, *dz));
                        send(sender, ZpMsg::SetDeadzoneOn(s, *dz));
                    },
                }
            }

            if let ZettOpts { deadzone_off: Some(dz), .. } = &opts {
                match input {
                    Single(trigger) => {
                        send(sender, ZpMsg::SetDeadzoneOff(trigger, *dz));
                    },
                    Axis(axis) => {
                        send(sender, ZpMsg::SetDeadzoneOff(axis, *dz));
                    },
                    Coords(ax, ay) => {
                        send(sender, ZpMsg::SetDeadzoneOff(ax, *dz));
                        send(sender, ZpMsg::SetDeadzoneOff(ay, *dz));
                    },
                    Quartet(w, e, n, s) => {
                        send(sender, ZpMsg::SetDeadzoneOff(w, *dz));
                        send(sender, ZpMsg::SetDeadzoneOff(e, *dz));
                        send(sender, ZpMsg::SetDeadzoneOff(n, *dz));
                        send(sender, ZpMsg::SetDeadzoneOff(s, *dz));
                    },
                }
            }
        }
    }
}
