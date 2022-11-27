#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    I32(i32),
    F64(f64),
    Bool(bool),
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

impl Value {
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            _ => panic!("expected boolean, got not that"),
        }
    }
}
