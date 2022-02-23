use crossbeam_channel::{Sender, Receiver};
use crate::zettpadder::{ZpMsg};
use crate::parsers::zett::{parse_line};
use rustyline::{Editor};

#[derive(Debug, Copy, Clone)]
pub enum CliMsg {
    MacroCreated(usize),
}

pub fn run(sender: Sender<ZpMsg>, receiver: Receiver<CliMsg>) {
    let mut prompt = Editor::<()>::new();
    println!("Press Ctrl-C twice to quit");
    loop {
        match prompt.readline("> ") {
            Ok(line) => {
                prompt.add_history_entry(line.trim());
                parse_line(&sender, &receiver, line);
            },
            _ => { break; },
        };
    }
}
