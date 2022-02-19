use std::io;
use crossbeam_channel::{Sender};
use crate::zettpadder::{ZpMsg};
use crate::parsers::zett::parse_line;

use rustyline::{Editor};

pub fn run(sender: Sender<ZpMsg>) {
    let mut prompt = Editor::<()>::new();
    println!("Press Ctrl-C twice to quit");
    loop {
        match prompt.readline("0> ") {
            Ok(line) => {
                prompt.add_history_entry(line.trim());
                parse_line(&sender, line);
            },
            _ => { break; },
        };
    }
}
