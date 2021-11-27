use std::thread::current;

pub struct Scanner {
    current: usize,
    tokens: Vec<Token>,
    source: String,
}

#[derive(Debug, Eq, PartialEq)]
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    token_type: TokenType,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        return Scanner {
            current: 0,
            tokens: Vec::new(),
            source,
        };
    }

    pub fn scan(&mut self) {
        while let Some(c) = self.source.chars().nth(self.current) {
            self.advance();
            match c {
                ';' => self.add_token(TokenType::Semicolon),
                '{' => self.add_token(TokenType::LeftBrace),
                '}' => self.add_token(TokenType::RightBrace),
                '(' => self.add_token(TokenType::LeftParen),
                ')' => self.add_token(TokenType::RightParen),
                ',' => self.add_token(TokenType::Comma),
                '.' => self.add_token(TokenType::Dot),
                '-' => self.add_token(TokenType::Minus),
                '+' => self.add_token(TokenType::Plus),
                '*' => self.add_token(TokenType::Star),
                '=' => self.add_double_token('=', TokenType::EqualEqual, TokenType::Equal),
                '!' => self.add_double_token('=', TokenType::BangEqual, TokenType::Bang),
                '>' => self.add_double_token('=', TokenType::GreaterEqual, TokenType::Greater),
                '<' => self.add_double_token('=', TokenType::LessEqual, TokenType::Less),
                '/' => {
                    let n = self.peek();
                    if n != Some('/') {
                        self.add_token(TokenType::Slash)
                    } else {
                        while self.peek() != Some('\n') {
                            self.advance();
                        }
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {}
                _ => {
                    println!("Unrecognised character {}", c);
                }
            }
        }
    }

    fn add_double_token(&mut self, next_char: char, lhs: TokenType, rhs: TokenType) {
        if let Some(c) = self.peek() {
            if c == next_char {
                self.advance();
                self.add_token(lhs);
            } else {
                self.add_token(rhs);
            }
        } else {
            self.add_token(rhs);
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }

    fn advance(&mut self) {
        self.current = self.current + 1;
    }

    fn add_token(&mut self, token: TokenType) {
        self.tokens.push(Token { token_type: token });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_char_tokens() {
        let input = String::from(";{}");
        let mut scanner = Scanner::new(input);
        scanner.scan();
        let tokens = scanner.tokens;
        let token_types: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(
            token_types,
            Vec::from([
                TokenType::Semicolon,
                TokenType::LeftBrace,
                TokenType::RightBrace
            ])
        );
    }

    #[test]
    fn double_char_tokens() {
        let input = String::from("==");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        assert_eq!(
            scanner.tokens.get(0).unwrap().token_type,
            TokenType::EqualEqual
        );

        let input = String::from("=");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        assert_eq!(scanner.tokens.get(0).unwrap().token_type, TokenType::Equal);
    }

    #[test]
    fn slash_tokens() {
        let input = String::from("// hello world \n");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        assert_eq!(scanner.tokens, []);

        let input = String::from("/");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        assert_eq!(scanner.tokens.get(0).unwrap().token_type, TokenType::Slash);
    }

    #[test]
    fn stress_test_with_simple_chars() {
        let input = String::from(
            "// this is a comment
(( )){} // grouping stuff
!*+-/=<> <= == // operators
",
        );
        let mut scanner = Scanner::new(input);
        scanner.scan();

        assert_eq!(scanner.tokens.len(), 16);
    }
}
