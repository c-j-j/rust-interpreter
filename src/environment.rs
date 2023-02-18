use crate::interpreter::{RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Debug)]
pub struct Environment {
    bindings: HashMap<String, Value>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        let bindings = HashMap::new();
        Rc::new(RefCell::new(Environment {
            bindings,
            enclosing: None,
        }))
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        let bindings = HashMap::new();
        Rc::new(RefCell::new(Environment {
            bindings,
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<Value> {
        let current = self.bindings.get(&name);

        if let Some(v) = current {
            return Some(v.clone());
        } else {
            if let Some(enclosing) = self.enclosing.as_ref() {
                return enclosing.borrow().get(name);
            } else {
                None
            }
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeError> {
        if self.bindings.contains_key(&name) {
            self.bindings.insert(name.clone(), value);
            Ok(())
        } else {
            match self.enclosing.as_mut() {
                None => Err(RuntimeError::UndefinedVariable(name)),
                Some(env) => env.borrow_mut().assign(name, value),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define() {
        let mut env = Environment::new();
        env.borrow_mut()
            .define(String::from("a"), Value::Number(1.0));
        assert_eq!(
            env.borrow().get(String::from("a")),
            Some(Value::Number(1.0))
        );
    }

    #[test]
    fn test_get() {
        let mut env = Environment::new();
        env.borrow_mut()
            .define(String::from("a"), Value::Number(1.0));
        assert_eq!(
            env.borrow().get(String::from("a")),
            Some(Value::Number(1.0))
        );
    }

    #[test]
    fn test_get_enclosing() {
        let mut env = Environment::new();
        env.borrow_mut()
            .define(String::from("a"), Value::Number(1.0));
        let mut env2 = Environment::new_with_enclosing(env);
        assert_eq!(
            env2.borrow().get(String::from("a")),
            Some(Value::Number(1.0))
        );
    }

    #[test]
    fn test_assign_enclosing() {
        let mut env = Environment::new();
        env.borrow_mut()
            .define(String::from("a"), Value::Number(1.0));
        let mut env2 = Environment::new_with_enclosing(env.clone());
        env2.borrow_mut()
            .assign(String::from("a"), Value::Number(2.0))
            .unwrap();
        assert_eq!(
            env2.borrow().get(String::from("a")),
            Some(Value::Number(2.0))
        );
    }

    #[test]
    fn test_assign_enclosing_undefined() {
        let env = Environment::new();
        env.borrow_mut()
            .define(String::from("a"), Value::Number(1.0));
        let env2 = Environment::new_with_enclosing(env.clone());
        env2.borrow_mut()
            .define(String::from("b"), Value::Number(2.0));
        assert_eq!(
            env2.borrow().get(String::from("b")),
            Some(Value::Number(2.0))
        );
        assert_eq!(
            env2.borrow().get(String::from("a")),
            Some(Value::Number(1.0))
        );
        assert_eq!(
            env.borrow().get(String::from("a")),
            Some(Value::Number(1.0))
        );
        let result = env2
            .borrow_mut()
            .assign(String::from("a"), Value::Number(3.0));
        assert!(result.is_ok());
        assert_eq!(
            env.borrow().get(String::from("a")),
            Some(Value::Number(3.0))
        );
    }

    #[test]
    fn test_assign() {
        let mut env = Environment::new();
        env.borrow_mut()
            .define(String::from("a"), Value::Number(1.0));
        env.borrow_mut()
            .assign(String::from("a"), Value::Number(2.0))
            .unwrap();
        assert_eq!(
            env.borrow().get(String::from("a")),
            Some(Value::Number(2.0))
        );
    }
}
