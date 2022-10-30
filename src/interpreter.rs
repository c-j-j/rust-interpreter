use crate::environment::Environment;
use crate::parser::{BinaryOperator, Expr, LiteralValue, Statement};

#[derive(PartialEq, Debug)]
pub enum RuntimeError {
    Runtime,
    UndefinedVariable(String),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

pub struct Interpreter<'a> {
    env: Environment<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
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
        &mut self,
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

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
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
                    None => Ok(Value::Nil),
                    Some(value) => {
                        let v = value.clone();
                        Ok(v)
                    }
                };
            }
            Expr::Assignment(name, expr) => match self.evaluate_expression(expr) {
                Ok(value) => self
                    .env
                    .assign(String::from(name), value)
                    .map(|_| Value::Nil),
                Err(err) => {
                    return Err(err);
                }
            },
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
                Ok(value) => match value {
                    Value::Number(num) => println!("{}", num),
                    Value::String(str) => println!("{}", str),
                    Value::Bool(bool) => println!("{}", bool),
                    Value::Nil => println!("nil"),
                },
                Err(err) => return Err(err),
            },
            Statement::Declaration(name, expr) => {
                let name = String::from_utf8(name.lexeme.clone()).unwrap();

                println!("Declaring {:?}", name);

                return match expr {
                    None => {
                        self.env.define(name, Value::Nil);
                        Ok(())
                    }
                    Some(expr) => match self.evaluate_expression(expr) {
                        Ok(value) => {
                            self.env.define(name, value);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::scanner;

    #[test]
    fn test_interpreter_assignment() {
        let input = "
        var a = 4;
        print a;";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&statements);
        assert_eq!(result, Ok(()));
    }
}
