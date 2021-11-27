mod scanner;
use scanner::Scanner;

fn main() {
    let mut scanner = Scanner::new(String::from("Hello"));
    let tokens = scanner.scan();
    print!("{:?}", tokens);
}
