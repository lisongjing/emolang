use std::io::{self, Write};

use crate::lexer::Lexer;


pub fn start() {
    loop {
        print!(">> ");
        io::stdout().flush().expect("Cannot write to console output");
        let mut line = String::new();
        io::stdin().read_line(&mut line).expect("Cannot read from console input");
        for token in Lexer::new(&line).tokenize().iter() {
            println!("{token:?}");
        }
    }
}