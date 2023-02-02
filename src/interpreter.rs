use crate::environment::Environment;
use crate::parser::{BinaryOperator, Expr, LiteralValue, Statement};
use std::fmt::Formatter;

#[derive(PartialEq, Debug)]
pub enum RuntimeError {
    Runtime,
    InvalidFunction,
    UndefinedVariable(String),
}

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub callable: fn(args: &[Value]) -> Result<Value, RuntimeError>,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}()", self.name)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    NativeFunction(NativeFunction),
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.define(
            String::from("clock"),
            Value::NativeFunction(NativeFunction {
                name: String::from("clock"),
                callable: |_| {
                    Ok(Value::Number(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as f64,
                    ))
                },
            }),
        );

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
            (Value::Number(a), BinaryOperator::Greater, Value::Number(b)) => {
                return Ok(Value::Bool(a > b))
            }
            (Value::Number(a), BinaryOperator::GreaterEqual, Value::Number(b)) => {
                return Ok(Value::Bool(a >= b))
            }
            (Value::Number(a), BinaryOperator::Less, Value::Number(b)) => {
                return Ok(Value::Bool(a < b))
            }
            (Value::Number(a), BinaryOperator::LessEqual, Value::Number(b)) => {
                return Ok(Value::Bool(a <= b))
            }
            (Value::Number(a), BinaryOperator::EqualEqual, Value::Number(b)) => {
                return Ok(Value::Bool(a == b))
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
            Expr::Call(expr, args) => {
                let callee = self.evaluate_expression(expr)?;

                let mut arguments = Vec::new();
                for arg in args {
                    let value = self.evaluate_expression(arg)?;
                    arguments.push(value);
                }

                match callee {
                    Value::NativeFunction(fun) => (fun.callable)(arguments.as_slice()),
                    _ => return Err(RuntimeError::InvalidFunction),
                }
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
                Ok(value) => match value {
                    Value::Number(num) => println!("{}", num),
                    Value::String(str) => println!("{}", str),
                    Value::Bool(bool) => println!("{}", bool),
                    Value::Nil => println!("nil"),
                    Value::NativeFunction(native_function) => {
                        println!("Function: {}", native_function.name)
                    }
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
                        Err(runtime_error) => Err(runtime_error),
                    },
                };
            }
            Statement::Block(statements) => {
                let env = Environment::new_with_enclosing(self.env.clone());
                self.env = env;
                match self.evaluate(statements) {
                    Ok(result) => {
                        self.env = *self.env.enclosing.clone().unwrap();
                        return Ok(result);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate_expression(condition)?;

                if let Value::Bool(true) = condition {
                    return self.evaluate_statement(then_branch);
                } else if let Some(else_branch) = else_branch {
                    return self.evaluate_statement(else_branch);
                }
            }
        }
        Ok(())
    }
}

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

    #[test]
    fn test_blocks() {
        let input = "
        var a = 4;
        {
            var a = 5;
            print a;
        }
        print a;";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&statements);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_if() {
        let input = "
        var a = 4;
        if (a == 4) {
            print a;
        }
        ";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&statements);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_clock() {
        let input = "
        print clock();
        ";
        let tokens = scanner::scan(String::from(input));
        let statements = parse(tokens).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&statements);
        assert_eq!(result, Ok(()));
    }
}
