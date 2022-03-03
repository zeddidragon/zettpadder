use crossbeam_channel::{Sender};
use crate::zettpadder::{ZpMsg};
use crate::parsers::zett::{parse_line};
use rustyline::{Editor};

pub fn run(sender: Sender<ZpMsg>) {
    let mut prompt = Editor::<()>::new();
    println!("Press Ctrl-C to exit");
    loop {
        match prompt.readline("> ") {
            Ok(line) => {
                prompt.add_history_entry(line.trim());
                parse_line(&sender, line);
            },
            _ => { break; },
        };
    }
    parse_line(&sender, "exit".to_string());
}
