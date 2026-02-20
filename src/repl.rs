use std::io::{self, Write};

use crate::{evaluator::eval, lexer::Lexer, parser::Parser, types::Environment};

pub fn start() {
    loop {
        print!(">> ");
        io::stdout()
            .flush()
            .expect("Cannot write to console output");
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Cannot read from console input");
        let mut lexer = Lexer::new(&line);
        let mut parser = Parser::new(&mut lexer);
        let mut env = Environment::new();
        let program = parser.parse_program();

        if !parser.errors().is_empty() {
            println!("Paser errors:\n\t{}", parser.errors().join("\n\t"));
            continue;
        }

        match eval(program, &mut env) {
            Ok(evaluated) => println!("{}", evaluated.inspect()),
            Err(error) => println!("Evaluator error:\n\t{error}"),
        }
    }
}
