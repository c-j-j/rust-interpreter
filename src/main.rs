mod interpreter;
mod parser;
mod scanner;

use crate::interpreter::Interpreter;
use std::io::Write;
use std::{env, io};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = args.get(1);

    if filepath.is_none() {
        repl();
    } else {
        run_file(filepath.unwrap());
    }
}

fn run_file(filepath: &String) {
    let contents =
        std::fs::read_to_string(filepath).expect("Something went wrong reading the file");
    let mut interpretter = Interpreter::new();

    run(contents, &mut interpretter);
}

fn repl() {
    let mut interpretter = Interpreter::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        run(buffer, &mut interpretter)
    }
}

fn run(buffer: String, interpretter: &mut Interpreter) {
    let tokens = scanner::scan(buffer);

    match parser::parse(tokens) {
        Ok(statements) => match interpretter.evaluate(&statements) {
            Err(runtime_err) => println!("runtime error {:?}", runtime_err),
            _ => {
                println!()
            }
        },
        Err(parse_errors) => {
            for parse_error in parse_errors {
                let formatted_lexeme = String::from_utf8(parse_error.token.lexeme.clone()).unwrap();
                println!(
                    "{:?}: {:?} Line {:} column {:}",
                    parse_error.error_type,
                    formatted_lexeme,
                    parse_error.token.line,
                    parse_error.token.column
                );
            }
        }
    }
}
