mod scanner;
use scanner::Scanner;
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

        println!("{:?}", tokens);
    }
}
