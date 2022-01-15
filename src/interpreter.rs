use crate::parser::{parse, BinaryOperator, Expr, LiteralValue};
use crate::scanner;

#[derive(PartialEq, Debug)]
enum RuntimeError {
    Runtime,
}

#[derive(PartialEq, Debug)]
enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

fn execute(expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Binary(left, right, op) => {
            let l = execute(left)?;
            let r = execute(right)?;

            match (l, op, r) {
                (Value::Number(a), BinaryOperator::Plus, Value::Number(b)) => {
                    return Ok(Value::Number(a + b))
                }
                _ => Err(RuntimeError::Runtime),
            }
        }
        Expr::Unary(expr, op) => {
            return Err(RuntimeError::Runtime);
        }
        Expr::Literal(lit) => match lit {
            LiteralValue::Number(num) => Ok(Value::Number(*num)),
            LiteralValue::String(str) => Ok(Value::String(str.clone())),
            LiteralValue::Boolean(bool) => Ok(Value::Bool(*bool)),
            LiteralValue::Nil => Ok(Value::Nil),
        },
    }
}

#[test]
fn test_parser_with_executer() {
    let input = "4 + 5 + 6";
    let tokens = scanner::scan(String::from(input));
    let expr = parse(tokens).unwrap();
    let result = execute(&expr);
    assert_eq!(result, Ok(Value::Number(15.)));
}
