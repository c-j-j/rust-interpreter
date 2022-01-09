use crate::scanner;
use crate::scanner::{Token, TokenType};
use std::any::Any;
use std::fmt::{Binary, Error};

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Box<Expr>, BinaryOperator),
    Unary(UnaryOperator, Box<Expr>),
    Literal(LiteralValue),
}

#[derive(Debug, Eq, PartialEq)]
enum BinaryOperator {
    EqualsEquals,
    Minus,
    Plus,
    Slash,
    Star,
    BangEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Eq, PartialEq)]
enum UnaryOperator {
    Bang,
    Plus,
}

#[derive(Debug, Eq, PartialEq)]
enum LiteralValue {
    Number(i32),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Eq, PartialEq)]
enum ErrorType {
    InvalidBinaryOperator,
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError {
    error_type: ErrorType,
    failed_token_type: TokenType,
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(tokens);

    parser.parse()
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        match self.comparison() {
            Ok(left) => {
                let mut expr = left;
                while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
                    let operator = self.previous();
                    match parse_binary_operator(operator) {
                        Ok(binary_op) => match self.comparison() {
                            Ok(right) => {
                                expr = Expr::Binary(Box::new(expr), Box::new(right), binary_op);
                            }
                            Err(err_right) => return Err(err_right),
                        },
                        Err(err) => return Err(err),
                    }
                }
                Ok(expr)
            }
            Err(left_err) => return Err(left_err),
        }
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        return match self.term() {
            Ok(left) => {
                let mut expr = left;
                while self.match_token(&[
                    TokenType::Greater,
                    TokenType::GreaterEqual,
                    TokenType::Less,
                    TokenType::LessEqual,
                ]) {
                    let operator = self.previous();
                    match parse_binary_operator(operator) {
                        Ok(binary_op) => {
                            match self.term() {
                                Ok(right) => {
                                    expr = Expr::Binary(Box::new(expr), Box::new(right), binary_op);
                                }
                                Err(right_err) => return Err(right_err),
                            };
                        }
                        Err(err) => return Err(err),
                    }
                }
                Ok(expr)
            }
            Err(left_err) => Err(left_err),
        };
    }

    fn match_token(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) {
        self.current = self.current + 1;
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().map(|t| t.token_type) == Some(*token_type)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn term(&self) -> Result<Expr, ParseError> {
        todo!()
    }

    fn previous(&self) -> TokenType {
        todo!()
    }

    fn is_at_end(&self) -> bool {
        return self.peek().map(|t| t.token_type) == Some(TokenType::EOF);
    }
}

fn parse_binary_operator(token_type: TokenType) -> Result<BinaryOperator, ParseError> {
    match token_type {
        TokenType::Minus => Ok(BinaryOperator::Minus),
        TokenType::Plus => Ok(BinaryOperator::Plus),
        TokenType::Slash => Ok(BinaryOperator::Slash),
        TokenType::Star => Ok(BinaryOperator::Star),
        TokenType::BangEqual => Ok(BinaryOperator::BangEqual),
        TokenType::EqualEqual => Ok(BinaryOperator::EqualEqual),
        TokenType::Greater => Ok(BinaryOperator::Greater),
        TokenType::GreaterEqual => Ok(BinaryOperator::GreaterEqual),
        TokenType::Less => Ok(BinaryOperator::Less),
        TokenType::LessEqual => Ok(BinaryOperator::LessEqual),
        _ => Err(ParseError {
            error_type: ErrorType::InvalidBinaryOperator,
            failed_token_type: token_type,
        }),
    }
}

#[test]
fn test_parser() {
    let input = "3==4";
    let tokens = scanner::scan(String::from(input));
    let expr = parse(tokens);
    assert_eq!(
        expr,
        Ok(Expr::Binary(
            Box::new(Expr::Literal(LiteralValue::Number(3))),
            Box::new(Expr::Literal(LiteralValue::Number(4))),
            BinaryOperator::EqualsEquals
        ))
    )
}
