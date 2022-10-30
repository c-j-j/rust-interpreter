use crate::interpreter::{RuntimeError, Value};
use std::collections::HashMap;

pub struct Environment<'a> {
    bindings: HashMap<String, Value>,
    enclosing: Option<Box<&'a Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        let bindings = HashMap::new();
        Environment {
            bindings,
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: &'a Environment<'_>) -> Self {
        let bindings = HashMap::new();
        Environment {
            bindings,
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<&Value> {
        self.bindings
            .get(&name)
            .or(self.enclosing.as_ref().and_then(|env| env.get(name)))
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeError> {
        if self.bindings.contains_key(&name) {
            self.bindings.insert(name.clone(), value);
            Ok(())
        } else {
            let enclosing = &self.enclosing;
            match enclosing {
                Some(env) => env.assign(name, value), // TODO carry on from here, how do we get the enclosing env and mutate?
                None => Err(RuntimeError::UndefinedVariable(name)),
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
        env.define(String::from("a"), Value::Number(1.0));
        assert_eq!(env.get(String::from("a")), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_get() {
        let mut env = Environment::new();
        env.define(String::from("a"), Value::Number(1.0));
        assert_eq!(env.get(String::from("a")), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_get_enclosing() {
        let mut env = Environment::new();
        env.define(String::from("a"), Value::Number(1.0));
        let mut env2 = Environment::new_with_enclosing(&env);
        assert_eq!(env2.get(String::from("a")), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_assign() {
        let mut env = Environment::new();
        env.define(String::from("a"), Value::Number(1.0));
        env.assign(String::from("a"), Value::Number(2.0)).unwrap();
        assert_eq!(env.get(String::from("a")), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_assign_enclosing() {
        let mut env = Environment::new();
        env.define(String::from("a"), Value::Number(1.0));
        let mut env2 = Environment::new_with_enclosing(&env);
        env2.assign(String::from("a"), Value::Number(2.0)).unwrap();
        assert_eq!(env2.get(String::from("a")), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_assign_enclosing_undefined() {
        let mut env = Environment::new();
        env.define(String::from("a"), Value::Number(1.0));
        let mut env2 = Environment::new_with_enclosing(&env);
        env2.define(String::from("b"), Value::Number(2.0));
        assert_eq!(env2.get(String::from("b")), Some(&Value::Number(2.0)));
        assert_eq!(env2.get(String::from("a")), Some(&Value::Number(1.0)));
    }
}
