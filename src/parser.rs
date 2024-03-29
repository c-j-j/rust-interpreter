use crate::scanner::{Literal, Token, TokenType};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Box<Expr>, BinaryOperator),
    Unary(Box<Expr>, UnaryOperator),
    Literal(LiteralValue),
    Variable(Token),
    Assignment(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expr),
    Print(Expr),
    Declaration(Token, Option<Expr>),
    Block(Vec<Statement>),
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        block: Vec<Statement>,
    },
    Return(Token, Option<Expr>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::Slash => write!(f, "/"),
            BinaryOperator::Star => write!(f, "*"),
            BinaryOperator::BangEqual => write!(f, "!="),
            BinaryOperator::EqualEqual => write!(f, "=="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::And => write!(f, "and"),
            BinaryOperator::Or => write!(f, "or"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorType {
    InvalidBinaryOperator,
    InvalidUnaryOperator,
    UnexpectedCharacter,
    InvalidAssignmentTarget,
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub error_type: ErrorType,
    pub token: Token,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, Vec<ParseError>> {
    let mut parser = Parser::new(tokens);

    parser.parse()
}

fn lexeme_to_name(var_token: &Token) -> String {
    String::from_utf8(var_token.lexeme.clone()).unwrap()
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    fn parse(&mut self) -> Result<Vec<Statement>, Vec<ParseError>> {
        let mut statements: Vec<Statement> = vec![];
        let mut errors: Vec<ParseError> = vec![];

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }
        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::Fun]) {
            return self.function_declaration();
        }
        if self.match_token(&[TokenType::Var]) {
            return self.declaration_statement();
        }
        self.statement()
    }

    fn function_declaration(&mut self) -> Result<Statement, ParseError> {
        let name_token = self.consume(TokenType::Identifier)?;

        self.consume(TokenType::LeftParen)?;
        let mut params: Vec<Token> = vec![];
        if !self.check(&TokenType::RightParen) {
            loop {
                match self.consume(TokenType::Identifier) {
                    Ok(param) => {
                        params.push(param);
                    }
                    Err(err) => return Err(err),
                }
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen)?;
        self.consume(TokenType::LeftBrace)?;
        let mut statements: Vec<Statement> = vec![];
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(err) => return Err(err),
            }
        }
        self.consume(TokenType::RightBrace)?;
        Ok(Statement::Function {
            name: name_token,
            params,
            block: statements,
        })
    }

    fn declaration_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenType::Identifier).and_then(|token| {
            let mut initialiser: Option<Expr> = None;
            if self.match_token(&[TokenType::Equal]) {
                match self.expression() {
                    Ok(expr) => {
                        initialiser = Some(expr);
                    }
                    Err(err) => return Err(err),
                }
            }

            self.consume(TokenType::Semicolon)
                .map(|_| Statement::Declaration(token, initialiser))
        })
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return self.block_statement();
        }
        return self.expr_statement();
    }

    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        let keyword = self.previous_token().clone();
        let mut value: Option<Expr> = None;
        if !self.check(&TokenType::Semicolon) {
            match self.expression() {
                Ok(expr) => {
                    value = Some(expr);
                }
                Err(err) => return Err(err),
            }
        }
        self.consume(TokenType::Semicolon)?;
        Ok(Statement::Return(keyword, value))
    }

    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenType::LeftParen)
            .and_then(|_| self.expression())
            .and_then(|condition| match self.consume(TokenType::RightParen) {
                Ok(_) => {
                    let then_branch = self.statement();
                    if then_branch.is_err() {
                        return Err(then_branch.err().unwrap());
                    }
                    let mut else_branch: Option<Statement> = None;
                    if self.match_token(&[TokenType::Else]) {
                        else_branch = Some(self.statement()?);
                    }
                    Ok(Statement::If {
                        condition,
                        then_branch: Box::new(then_branch?),
                        else_branch: else_branch.map(Box::new),
                    })
                }
                Err(err) => return Err(err),
            })
    }

    fn block_statement(&mut self) -> Result<Statement, ParseError> {
        let mut statements: Vec<Statement> = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(next_declaration) => {
                    statements.push(next_declaration);
                }
                Err(err) => return Err(err),
            }
        }

        // there is a bug that is causing the following to fail - seems that the token has already been consumed
        if let Err(err) = self.consume(TokenType::RightBrace) {
            return Err(err);
        }

        Ok(Statement::Block(statements))
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.expression().and_then(|expr| {
            self.consume(TokenType::Semicolon)
                .map(|_| Statement::Print(expr))
        })
    }

    fn expr_statement(&mut self) -> Result<Statement, ParseError> {
        self.expression().and_then(|expr| {
            self.consume(TokenType::Semicolon)
                .map(|_| Statement::Expression(expr))
        })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        return match self.equality() {
            Ok(equality_expr) => {
                if self.match_token(&[TokenType::Equal]) {
                    let equals = self.previous_token().clone();
                    return match self.assignment() {
                        Ok(assignment_expr) => match equality_expr {
                            Expr::Variable(var_token) => {
                                let name = lexeme_to_name(&var_token);
                                Ok(Expr::Assignment(name, Box::new(assignment_expr)))
                            }
                            _ => Err(ParseError {
                                error_type: ErrorType::InvalidAssignmentTarget,
                                token: equals.clone(),
                            }),
                        },
                        Err(assignment_err) => Err(assignment_err),
                    };
                }
                Ok(equality_expr)
            }
            Err(error) => Err(error),
        };
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        match self.comparison() {
            Ok(left) => {
                let mut expr = left;
                while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
                    let operator_token = self.previous_token();
                    match parse_binary_operator(operator_token) {
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
                    let operator = self.previous_token();
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
                    let operator = self.previous_token();
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
                    let operator = self.previous_token();
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
            let operator_token = self.previous_token().clone();
            return match self.unary() {
                Ok(expr) => match parse_unary_operator(&operator_token) {
                    Ok(unary_op) => Ok(Expr::Unary(Box::new(expr), unary_op)),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            };
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        return self
            .consume(TokenType::RightParen)
            .and_then(|_| Ok(Expr::Call(Box::new(expr), arguments)));
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

        let last = self.peek().expect("No token found");
        Err(ParseError {
            error_type: ErrorType::UnexpectedCharacter,
            token: last.clone(),
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
            let next_token = next.clone();
            if next.token_type == token_type {
                self.advance();
                return Ok(next_token);
            } else {
                self.synchronize();
                return Err(ParseError {
                    error_type: ErrorType::UnexpectedCharacter,
                    token: next_token,
                });
            }
        }

        return Err(ParseError {
            error_type: ErrorType::UnexpectedCharacter,
            token: Token {
                token_type: TokenType::EOF,
                lexeme: [].to_vec(),
                line: 0,
                literal: None,
                column: 0,
            },
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

    fn is_at_end(&self) -> bool {
        self.peek().map(|t| t.token_type) == Some(TokenType::EOF)
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous_token().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().unwrap().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }
}

fn parse_binary_operator(token: &Token) -> Result<BinaryOperator, ParseError> {
    match token.token_type {
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
            token: token.clone(),
        }),
    }
}

fn parse_unary_operator(token: &Token) -> Result<UnaryOperator, ParseError> {
    match token.token_type {
        TokenType::Minus => Ok(UnaryOperator::Minus),
        TokenType::Bang => Ok(UnaryOperator::Bang),
        _ => Err(ParseError {
            error_type: ErrorType::InvalidUnaryOperator,
            token: token.clone(),
        }),
    }
}

#[allow(dead_code)]
fn print_ast_expr(expr: &Expr) -> String {
    match expr {
        Expr::Binary(left, right, op) => {
            let l = print_ast_expr(left);
            let r = print_ast_expr(right);
            let oper = print_binary_op(op);
            return format!("({} {} {})", oper, l, r);
        }
        Expr::Unary(expr, op) => {
            let l = print_ast_expr(expr);
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
        Expr::Assignment(name, value) => return format!("{} = {}", name, print_ast_expr(value)),
        Expr::Call(expr, args) => {
            let mut arg_str = String::new();
            for arg in args {
                arg_str.push_str(&print_ast_expr(arg));
                arg_str.push_str(", ");
            }
            return format!("{}({})", print_ast_expr(expr), arg_str);
        }
    }
}

#[allow(dead_code)]
fn print_ast(statement: &Statement) -> String {
    match statement {
        Statement::Expression(expr) => print_ast_expr(expr),
        Statement::Print(expr) => format!("print {}", print_ast_expr(expr)),
        Statement::Declaration(name, expr) => match expr {
            None => {
                format!("var {};", lexeme_to_name(name))
            }
            Some(value) => {
                format!("var {} = {}", lexeme_to_name(name), print_ast_expr(value))
            }
        },
        Statement::Block(statements) => print_block_ast(statements),
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut result = String::from("if (");
            result.push_str(&print_ast_expr(condition));
            result.push_str(") ");
            result.push_str(&print_ast(then_branch));
            if let Some(else_branch) = else_branch {
                result.push_str(" else ");
                result.push_str(&print_ast(else_branch));
            }
            result
        }
        Statement::Function {
            name,
            params,
            block,
        } => {
            let mut result = String::from("fun ");
            result.push_str(&lexeme_to_name(name));
            result.push_str("(");
            for param in params {
                result.push_str(&lexeme_to_name(param));
                result.push_str(", ");
            }
            result.push_str(") ");
            result.push_str(&print_block_ast(block));
            result
        }
        Statement::Return(_, _) => String::from("return"),
    }
}

fn print_block_ast(statements: &Vec<Statement>) -> String {
    let mut result = String::from("{");
    for statement in statements {
        result.push_str(&print_ast(statement));
        result.push_str(";");
    }
    result.push_str("}");
    result
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::scanner;

    #[test]
    fn test_declaration() {
        let input = "var a = 3;";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        let statement = statements.first().unwrap();
        assert_eq!(print_ast(statement), "var a = 3");
    }

    #[test]
    fn test_assignment() {
        let input = "a = 3;";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        let statement = statements.first().unwrap();
        assert_eq!(print_ast(statement), "a = 3");
    }

    #[test]
    fn test_blocks() {
        let input = "{ var a = 3; print a; }";
        let tokens = scanner::scan(String::from(input));

        let statements = parse(tokens).unwrap();
        let statement = statements.first().unwrap();
        assert_eq!(print_ast(statement), "{var a = 3;print a;}");
    }

    #[test]
    fn test_parser_with_declaration_statement() {
        let input = "var a = 3;";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        assert_eq!(statements.len(), 1);
        let statement = &statements[0];
        match statement {
            Statement::Declaration(
                Token {
                    token_type,
                    lexeme,
                    literal: _,
                    line: _,
                    column: _,
                },
                expr,
            ) => {
                assert_eq!(token_type, &TokenType::Identifier);
                assert_eq!(lexeme, b"a");

                match expr {
                    Some(Expr::Literal(LiteralValue::Number(num))) => {
                        assert_eq!(num, &3.0);
                    }
                    _ => panic!("Expected literal expression"),
                }
            }
            _ => panic!("Expected declaration statement"),
        }
    }

    #[test]
    fn test_parser_with_fun_declaration_statement() {
        let input = "fun a() { print 3; }";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        assert_eq!(statements.len(), 1);
        let statement = &statements[0];
        match statement {
            Statement::Function {
                name,
                params,
                block,
            } => {
                assert_eq!(name.lexeme, b"a");
                assert_eq!(params.len(), 0);
                assert_eq!(block.len(), 1);
                let print_statement = &block[0];
                match print_statement {
                    Statement::Print(expr) => match expr {
                        Expr::Literal(LiteralValue::Number(num)) => {
                            assert_eq!(num, &3.0);
                        }
                        _ => panic!("Expected literal expression"),
                    },
                    _ => panic!("Expected print statement"),
                }
            }
            _ => panic!("Expected function declaration statement"),
        }
    }
}
