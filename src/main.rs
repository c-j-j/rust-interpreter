mod interpreter;
mod parser;
mod scanner;

use crate::interpreter::evaluate;
use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer);
        if buffer.trim().is_empty() {
            break;
        }
        let tokens = scanner::scan(buffer);
        match parser::parse(tokens) {
            Ok(statements) => match evaluate(&statements) {
                Ok(_) => {
                    println!("successfully evaluated")
                }
                Err(runtime_err) => println!("runtime error {:?}", runtime_err),
            },
            Err(parse_error) => {
                println!("parse error {:?}", parse_error)
            }
        }
    }
}
