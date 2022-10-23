use crate::parser::{parse, BinaryOperator, Expr, LiteralValue, Statement};
use crate::scanner;
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum RuntimeError {
    Runtime,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

pub fn evaluate(statements: &Vec<Statement>) -> Result<(), RuntimeError> {
    let mut interpretter = Interpreter::new();
    interpretter.evaluate(statements)
}

struct Environment {
    bindings: HashMap<String, Value>,
}

impl Environment {
    fn new() -> Self {
        let hashmap = HashMap::new();
        Environment { bindings: hashmap }
    }

    fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    fn get(&self, name: String) -> Option<&Value> {
        self.bindings.get(name.as_str())
    }
}

struct Interpreter {
    env: Environment,
}

impl Interpreter {
    fn new() -> Self {
        let env = Environment::new();
        Interpreter { env }
    }

    pub fn evaluate(&mut self, statements: &Vec<Statement>) -> Result<(), RuntimeError> {
        for statement in statements {
            match self.evaluate_statement(&statement) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    fn evaluate_binary_op(
        &self,
        left: &Expr,
        right: &Expr,
        op: &BinaryOperator,
    ) -> Result<Value, RuntimeError> {
        let l = self.evaluate_expression(left)?;
        let r = self.evaluate_expression(right)?;

        match (l, op, r) {
            (Value::Number(a), BinaryOperator::Plus, Value::Number(b)) => {
                return Ok(Value::Number(a + b))
            }
            (Value::Number(a), BinaryOperator::Minus, Value::Number(b)) => {
                return Ok(Value::Number(a - b))
            }
            (Value::Number(a), BinaryOperator::Star, Value::Number(b)) => {
                return Ok(Value::Number(a * b))
            }
            (Value::Number(a), BinaryOperator::Slash, Value::Number(b)) => {
                return Ok(Value::Number(a / b))
            }
            (Value::Bool(a), BinaryOperator::And, Value::Bool(b)) => {
                return Ok(Value::Bool(a && b))
            }
            (Value::Bool(a), BinaryOperator::Or, Value::Bool(b)) => {
                return Ok(Value::Bool(a || b));
            }
            _ => Err(RuntimeError::Runtime),
        }
    }

    fn evaluate_expression(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Binary(left, right, op) => self.evaluate_binary_op(left, right, op),
            Expr::Unary(_expr, _op) => {
                todo!()
            }
            Expr::Literal(lit) => match lit {
                LiteralValue::Number(num) => Ok(Value::Number(*num)),
                LiteralValue::String(str) => Ok(Value::String(str.clone())),
                LiteralValue::Boolean(bool) => Ok(Value::Bool(*bool)),
                LiteralValue::Nil => Ok(Value::Nil),
            },
            Expr::Variable(token) => {
                let name = String::from_utf8(token.lexeme.clone()).unwrap();
                return match self.env.get(name) {
                    None => return Ok(Value::Nil),
                    Some(value) => {
                        let v = value.clone();
                        return Ok(v);
                    }
                };
            }
        }
    }

    fn evaluate_statement(&mut self, statement: &Statement) -> Result<(), RuntimeError> {
        match statement {
            Statement::Expression(expr) => {
                return match self.evaluate_expression(expr) {
                    Ok(_value) => Ok(()),
                    Err(err) => Err(err),
                }
            }
            Statement::Print(expr) => match self.evaluate_expression(expr) {
                Ok(value) => println!("{:?}", value),
                Err(err) => return Err(err),
            },
            Statement::Declaration(name, expr) => {
                let name = String::from_utf8(name.lexeme.clone()).unwrap();

                return match expr {
                    None => {
                        &self.env.define(name, Value::Nil);
                        Ok(())
                    }
                    Some(expr) => match self.evaluate_expression(expr) {
                        Ok(value) => {
                            &self.env.define(name, value);
                            Ok(())
                        }
                        Err(runtmime_error) => Err(runtmime_error),
                    },
                };
            }
        }
        Ok(())
    }
}

// #[test]
// fn test_parser_with_expr_evaluator() {
//     let input = "4 + 5 + 6";
//     let tokens = scanner::scan(String::from(input));
//     let expr = parse(tokens).unwrap();
//     let result = evaluate_expression(&expr);
//     assert_eq!(result, Ok(Value::Number(15.)));
// }

#[test]
fn test_parser_with_evaluator() {
    let input = "var a = (5 + 6) * 2; print a;";
    let tokens = scanner::scan(String::from(input));
    let statements = parse(tokens).unwrap();
    let result = evaluate(&statements);
    assert_eq!(result.is_ok(), true);
}
