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

fn parse_outputs(iter: &mut Iter<&str>) -> Mapping {
    let next = iter.next();
    if next.is_none() {
        println!("No assignment supplied, clearing bind for key");
        return Mapping::Noop
    }
    let next = next.unwrap();
    match next.to_lowercase().as_str() {
        "layer" => {
            let arg1 = iter.next().map(|v| v.parse::<u8>());
            if let Some(Ok(layer)) = arg1 {
                Mapping::Layer(layer)
            } else {
                println!("Urecognized layer: {:?}", arg1);
                Mapping::Noop
            }
        }
        _ => {
            parse_output(next)
        }
    }
}

fn parse_line(sender: &Sender<ZpMsg>, line: Result<String, Error>) {
    match line {
        Ok(line) => {
            let tokens = line
                .split('#') // Remove comments
                .nth(0)
                .unwrap()
                .trim_start() // Remove any indentation
                .split_whitespace()
                .collect::<Vec<_>>();
            let mut iter = tokens.iter();
            match iter.next() {
                Some(string) => {
                    let normalized = string
                        .clone()
                        .to_lowercase();
                    match normalized.as_str() {
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
                        "joyxy" => {
                        },
                        _ => {
                            let input = parse_input(&string.to_string());
                            if !input.is_ok() {
                                println!("Unknown command: {}", normalized);
                                return;
                            };
                            let input = input.unwrap() as u8;
                            let parsed = parse_outputs(&mut iter);
                            send(sender, ZpMsg::Bind(input, parsed));
                        }
                    }
                },
                None => {},
            }
        }
        Err(err) => {
            println!("Error parsing zett: {:?}", err);
        }
    }
}
