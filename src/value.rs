use crate::node::Node;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assign(String, Node),
    Return(Node),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    I32(i32),
    F64(f64),
    Bool(bool),
    Func(Func),
    Unit,
    Err,
}

impl Value {
    pub fn neg(&self) -> Value {
        match self {
            Value::I32(num) => Value::I32(-num),
            Value::F64(num) => Value::F64(-num),
            _ => Value::Err,
        }
    }

    pub fn add(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a + b),
            _ => Value::Err,
        }
    }

    pub fn sub(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a - b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a - b),
            _ => Value::Err,
        }
    }

    pub fn mul(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a * b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a * b),
            _ => Value::Err,
        }
    }

    pub fn div(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a / b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a / b),
            _ => Value::Err,
        }
    }

    pub fn eq(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::Bool(*a == b),
            (Value::F64(a), Value::F64(b)) => Value::Bool(*a == b),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a == b),
            _ => Value::Err,
        }
    }
}

#[derive(Default)]
pub struct Scope<'a> {
    vars: HashMap<String, Value>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn get(&self, name: &str) -> Value {
        if let Some(value) = self.vars.get(name) {
            return value.clone();
        } else if let Some(parent) = self.parent {
            return parent.get(name);
        } else {
            return Value::Err;
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

impl<'a> Scope<'a> {
    pub fn root(&self) -> Scope {
        if let Some(mut parent) = self.parent {
            while let Some(p) = parent.parent {
                parent = p
            }

            return Scope {
                vars: HashMap::new(),
                parent: Some(parent),
            };
        } else {
            return self.child();
        }
    }

    pub fn child(&self) -> Scope {
        return Scope {
            vars: HashMap::new(),
            parent: Some(self),
        };
    }
}
