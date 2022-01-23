use crate::parser::{parse, BinaryOperator, Expr, LiteralValue, Statement};
use crate::scanner;

#[derive(PartialEq, Debug)]
pub enum RuntimeError {
    Runtime,
}

#[derive(PartialEq, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

fn evaluate_binary_op(
    left: &Expr,
    right: &Expr,
    op: &BinaryOperator,
) -> Result<Value, RuntimeError> {
    let l = evaluate_expression(left)?;
    let r = evaluate_expression(right)?;

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
        (Value::Bool(a), BinaryOperator::And, Value::Bool(b)) => return Ok(Value::Bool(a && b)),
        (Value::Bool(a), BinaryOperator::Or, Value::Bool(b)) => {
            return Ok(Value::Bool(a || b));
        }
        _ => Err(RuntimeError::Runtime),
    }
}

pub fn evaluate_expression(expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Binary(left, right, op) => evaluate_binary_op(left, right, op),
        Expr::Unary(_expr, _op) => {
            todo!()
        }
        Expr::Literal(lit) => match lit {
            LiteralValue::Number(num) => Ok(Value::Number(*num)),
            LiteralValue::String(str) => Ok(Value::String(str.clone())),
            LiteralValue::Boolean(bool) => Ok(Value::Bool(*bool)),
            LiteralValue::Nil => Ok(Value::Nil),
        },
    }
}

pub fn evaluate_statement(statement: &Statement) -> Result<(), RuntimeError> {
    match statement {
        Statement::Expression(expr) => {
            return match evaluate_expression(expr) {
                Ok(_value) => Ok(()),
                Err(err) => Err(err),
            }
        }
        Statement::Print(expr) => match evaluate_expression(expr) {
            Ok(value) => println!("{:?}", value),
            Err(err) => return Err(err),
        },
    }
    Ok(())
}

pub fn evaluate(statements: &Vec<Statement>) -> Result<(), RuntimeError> {
    for statement in statements {
        match evaluate_statement(statement) {
            Ok(_) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
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
    let input = "print 4 + 5 + 6; print 1 + 5 + 6;";
    let tokens = scanner::scan(String::from(input));
    let statements = parse(tokens).unwrap();
    let result = evaluate(&statements);
    assert_eq!(result.is_ok(), true);
}
