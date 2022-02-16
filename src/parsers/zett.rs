use std::fs::File;
use std::io::{self, BufRead, Error};
use crate::function::{Function};
use crossbeam_channel::{Sender};
use crate::zettpadder::{ZpMsg};
use crate::mapping::{Mapping};

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
    
    let mut i = 0;
    for line in io::BufReader::new(file).lines() {
        parse_line(sender, line);
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
                            let arg1 = iter
                                .next()
                                .and_then(|s| s.parse::<u8>();
                            send(sender, ZpMsg::SetWriteLayer(layer));
                        },
                        _ => {
                            println!("Unknown action or command: {}", string);
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
