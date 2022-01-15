use crate::scanner;
use crate::scanner::{Literal, Token, TokenType};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Box<Expr>, BinaryOperator),
    Unary(Box<Expr>, UnaryOperator),
    Literal(LiteralValue),
}

#[derive(Debug, Eq, PartialEq)]
pub enum BinaryOperator {
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
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Eq, PartialEq)]
enum ErrorType {
    InvalidBinaryOperator,
    InvalidUnaryOperator,
    UnexpectedCharacter,
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
                    let operator = self.previous_token_type();
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
                    let operator = self.previous_token_type();
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

    fn term(&mut self) -> Result<Expr, ParseError> {
        return match self.factor() {
            Ok(left) => {
                let mut expr = left;
                while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
                    let operator = self.previous_token_type();
                    match parse_binary_operator(operator) {
                        Ok(binary_op) => {
                            match self.factor() {
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

    fn factor(&mut self) -> Result<Expr, ParseError> {
        return match self.unary() {
            Ok(left) => {
                let mut expr = left;
                while self.match_token(&[TokenType::Slash, TokenType::Star]) {
                    let operator = self.previous_token_type();
                    match parse_binary_operator(operator) {
                        Ok(binary_op) => {
                            match self.unary() {
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

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous_token_type();
            return match self.unary() {
                Ok(expr) => match parse_unary_operator(operator) {
                    Ok(unary_op) => Ok(Expr::Unary(Box::new(expr), unary_op)),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            };
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(LiteralValue::Boolean(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(LiteralValue::Boolean(true)));
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralValue::Nil));
        }

        if self.match_token(&[TokenType::Number]) {
            let number = self.previous_token();
            return match number.literal.as_ref().unwrap() {
                Literal::String(string) => Ok(Expr::Literal(LiteralValue::String(string.clone()))),
                Literal::Number(number) => Ok(Expr::Literal(LiteralValue::Number(*number))),
            };
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression();

            if let Err(err) = self.consume(TokenType::RightParen) {
                return Err(err);
            }
            return expr;
        }

        let last = self.peek();
        Err(ParseError {
            error_type: ErrorType::UnexpectedCharacter,
            failed_token_type: last.map(|t| t.token_type).unwrap_or(TokenType::EOF),
        })
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

    fn consume(&mut self, token_type: TokenType) -> Result<(), ParseError> {
        if self.peek().map(|t| t.token_type) == Some(token_type) {
            self.advance();
            return Ok(());
        }
        Err(ParseError {
            error_type: ErrorType::UnexpectedCharacter,
            failed_token_type: token_type,
        })
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

    fn previous_token(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn previous_token_type(&self) -> TokenType {
        self.previous_token().token_type
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

fn parse_unary_operator(token_type: TokenType) -> Result<UnaryOperator, ParseError> {
    match token_type {
        TokenType::Minus => Ok(UnaryOperator::Minus),
        TokenType::Bang => Ok(UnaryOperator::Bang),
        _ => Err(ParseError {
            error_type: ErrorType::InvalidUnaryOperator,
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
            Box::new(Expr::Literal(LiteralValue::Number(3.))),
            Box::new(Expr::Literal(LiteralValue::Number(4.))),
            BinaryOperator::EqualEqual
        ))
    )
}
