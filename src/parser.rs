use crate::scanner::{Literal, Token, TokenType};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Box<Expr>, BinaryOperator),
    Unary(Box<Expr>, UnaryOperator),
    Literal(LiteralValue),
    Variable(Token),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(Expr),
    Print(Expr),
    Declaration(Token, Option<Expr>),
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
    And,
    Or,
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

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut parser = Parser::new(tokens);

    parser.parse()
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements: Vec<Statement> = vec![];

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(err) => return Err(err),
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::Var]) {
            return self.declaration_statement();
        }
        return self.statement();
    }

    fn declaration_statement(&mut self) -> Result<Statement, ParseError> {
        match self.consume(TokenType::Identifier) {
            Ok(token) => {
                let mut initialiser: Option<Expr> = None;
                if self.match_token(&[TokenType::Equal]) {
                    match self.expression() {
                        Ok(expr) => {
                            initialiser = Some(expr);
                        }
                        Err(err) => return Err(err),
                    }
                }

                match self.consume(TokenType::Semicolon) {
                    Ok(_) => return Ok(Statement::Declaration(token, initialiser)),
                    Err(parse_error) => Err(parse_error),
                }
            }
            Err(err) => Err(err),
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }
        return self.expr_statement();
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        return match self.expression() {
            Ok(expr) => match self.consume(TokenType::Semicolon) {
                Ok(_) => Ok(Statement::Print(expr)),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        };
    }

    fn expr_statement(&mut self) -> Result<Statement, ParseError> {
        return match self.expression() {
            Ok(expr) => match self.consume(TokenType::Semicolon) {
                Ok(_) => Ok(Statement::Expression(expr)),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        };
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
        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous_token().clone()));
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

    fn consume(&mut self, token_type: TokenType) -> Result<Token, ParseError> {
        if let Some(next) = self.peek() {
            if next.token_type == token_type {
                let token = next.clone();
                self.advance();
                return Ok(token);
            } else {
                return Err(ParseError {
                    error_type: ErrorType::UnexpectedCharacter,
                    failed_token_type: token_type,
                });
            }
        }

        return Err(ParseError {
            error_type: ErrorType::UnexpectedCharacter,
            failed_token_type: TokenType::EOF,
        });
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
        TokenType::And => Ok(BinaryOperator::And),
        TokenType::Or => Ok(BinaryOperator::Or),
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

fn print_ast(expr: &Expr) -> String {
    match expr {
        Expr::Binary(left, right, op) => {
            let l = print_ast(left);
            let r = print_ast(right);
            let oper = print_binary_op(op);
            return format!("({} {} {})", oper, l, r);
        }
        Expr::Unary(expr, op) => {
            let l = print_ast(expr);
            let oper = print_unary_op(op);
            return format!("{}{}", oper, l);
        }
        Expr::Literal(lit) => match lit {
            LiteralValue::Number(num) => num.to_string(),
            LiteralValue::String(str) => str.to_string(),
            LiteralValue::Boolean(bool) => bool.to_string(),
            LiteralValue::Nil => String::from("nil"),
        },
        Expr::Variable(v) => {
            return String::from_utf8(v.lexeme.clone()).unwrap();
        }
    }
}

fn print_binary_op(op: &BinaryOperator) -> &str {
    match op {
        BinaryOperator::Minus => "+",
        BinaryOperator::Plus => "-",
        BinaryOperator::Slash => "/",
        BinaryOperator::Star => "*",
        BinaryOperator::BangEqual => "!=",
        BinaryOperator::EqualEqual => "==",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::And => "and",
        BinaryOperator::Or => "or",
    }
}

fn print_unary_op(op: &UnaryOperator) -> &str {
    match op {
        UnaryOperator::Bang => "!",
        UnaryOperator::Minus => "-",
    }
}

// #[test]
// fn test_parser_with_formatter() {
//     let input = "3 > 4 + 1 * (1 + 2)";
//     let tokens = scanner::scan(String::from(input));
//     let statements = parse(tokens).unwrap();
//     let ast = print_ast(&expr);
//     assert_eq!(ast, "(> 3 (- 4 (* 1 (- 1 2))))")
// }
