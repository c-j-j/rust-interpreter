use std::collections::HashMap;

pub struct Scanner {
    current: usize,
    start: usize,
    tokens: Vec<Token>,
    source: Vec<u8>,
    line: u16,
    keywords: HashMap<String, TokenType>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Vec<u8>,
    pub line: u16,
    pub literal: Option<Literal>,
    pub column: usize,
}

pub fn scan(input: String) -> Vec<Token> {
    let mut scanner = Scanner::new(input);
    scanner.scan();

    scanner.tokens
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        let keywords: HashMap<String, TokenType> = vec![
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ]
        .into_iter()
        .map(|(k, v)| (String::from(k), v))
        .collect();

        return Scanner {
            current: 0,
            start: 0,
            tokens: Vec::new(),
            line: 0,
            source: source.into_bytes(),
            keywords,
        };
    }

    pub fn scan(&mut self) {
        while self.current < self.source.len() {
            self.scan_next();
        }
        self.add_token(TokenType::EOF);
    }

    fn scan_next(&mut self) {
        let c = self.source[self.current] as char;
        self.start = self.current;
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
            '"' => self.add_string_literal(),
            '/' => {
                let n = self.peek();
                if n != '/' {
                    self.add_token(TokenType::Slash)
                } else {
                    while self.peek() != '\n' {
                        self.advance();
                    }
                }
            }
            '\n' => self.line = self.line + 1,
            ' ' | '\t' | '\r' => {}
            _ => {
                if c.is_digit(10) {
                    self.add_number_literal();
                } else if c.is_alphanumeric() {
                    self.add_identifier();
                } else {
                    println!("Unrecognised character {}", c);
                }
            }
        }
    }

    fn add_double_token(&mut self, next_char: char, lhs: TokenType, rhs: TokenType) {
        if self.peek() == next_char {
            self.advance();
            self.add_token(lhs);
        } else {
            self.add_token(rhs);
        }
    }

    fn peek(&self) -> char {
        if self.current < self.source.len() {
            self.source[self.current] as char
        } else {
            '\0'
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 < self.source.len() {
            self.source[self.current + 1] as char
        } else {
            '\0'
        }
    }

    fn advance(&mut self) {
        self.current = self.current + 1;
    }

    fn add_token(&mut self, token: TokenType) {
        self.add_token_with_literal(token, None);
    }

    fn add_string_literal(&mut self) {
        while self.peek() != '"' {
            if self.peek() == '\n' {
                self.line = self.line + 1;
            }
            self.advance();
        }
        // advance after final "
        self.advance();

        let string = Literal::String(
            String::from_utf8(self.source[self.start + 1..self.current - 1].to_vec()).unwrap(),
        );
        self.add_token_with_literal(TokenType::String, Some(string));
    }

    fn add_number_literal(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance(); // consume .

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let num: f64 = self.get_current_string().parse().unwrap();
        let num_literal = Literal::Number(num);
        self.add_token_with_literal(TokenType::Number, Some(num_literal));
    }

    fn get_current_string(&mut self) -> String {
        String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap()
    }

    fn add_identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let str = self.get_current_string();
        let token = self.keywords.get(str.as_str()).clone();
        if let Some(t) = token {
            self.add_token(*t);
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn add_token_with_literal(&mut self, token: TokenType, literal: Option<Literal>) {
        let lexeme = self.source[self.start..self.current].to_vec();
        self.tokens.push(Token {
            lexeme,
            literal,
            token_type: token,
            line: self.line,
            column: self.start,
        });
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

        assert_eq!(scanner.tokens.len(), 0);

        let input = String::from("/");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        assert_eq!(scanner.tokens.get(0).unwrap().token_type, TokenType::Slash);
    }

    // // Having trouble with this test
    // #[test]
    // fn string_literal_tokens() {
    //     let input = String::from("\"hello\"");
    //     let mut scanner = Scanner::new(input);
    //     scanner.scan();
    //
    //     let token = scanner.tokens.get(0).unwrap();
    //     assert_eq!(token.token_type, TokenType::String);
    //     let expected_str = String::from("hello");
    //     assert!(matches!(
    //         token.literal.as_ref().unwrap(),
    //         Literal::String(expected_str)
    //     ));
    // }

    // Having trouble with this test
    #[test]
    fn number_literal_tokens() {
        let input = String::from("10.1234");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        let token = scanner.tokens.get(0).unwrap();
        assert_eq!(token.token_type, TokenType::Number);
    }

    #[test]
    fn identifier_reserved_tokens() {
        let input = String::from("while");
        let mut scanner = Scanner::new(input);
        scanner.scan();

        let token = scanner.tokens.get(0).unwrap();
        assert_eq!(token.token_type, TokenType::While);
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
