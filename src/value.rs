use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    I32(i32),
    F64(f64),
    Bool(bool),
    Tuple(Vec<Value>),
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
            (Value::Tuple(a), Value::Tuple(b)) => {
                if a.len() == b.len() {
                    let mut result = true;
                    for i in 0..a.len() {
                        match a[i].eq(b[i].clone()) {
                            Value::Err => return Value::Err,
                            Value::Bool(false) => result = false,
                            _ => {}
                        }
                    }
                    return Value::Bool(result);
                }
                return Value::Err;
            }
            _ => Value::Err,
        }
    }

    pub fn ne(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::Bool(*a != b),
            (Value::F64(a), Value::F64(b)) => Value::Bool(*a != b),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a != b),
            _ => Value::Err,
        }
    }

    pub fn lt(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::Bool(*a < b),
            (Value::F64(a), Value::F64(b)) => Value::Bool(*a < b),
            _ => Value::Err,
        }
    }

    pub fn le(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::Bool(*a <= b),
            (Value::F64(a), Value::F64(b)) => Value::Bool(*a <= b),
            _ => Value::Err,
        }
    }

    pub fn gt(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::Bool(*a > b),
            (Value::F64(a), Value::F64(b)) => Value::Bool(*a > b),
            _ => Value::Err,
        }
    }

    pub fn ge(&self, b: Value) -> Value {
        match (self, b) {
            (Value::I32(a), Value::I32(b)) => Value::Bool(*a >= b),
            (Value::F64(a), Value::F64(b)) => Value::Bool(*a >= b),
            _ => Value::Err,
        }
    }

    pub fn not(&self) -> Value {
        match self {
            Value::Bool(a) => Value::Bool(!*a),
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

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::I32(..) => Type::I32,
            Value::F64(..) => Type::F64,
            Value::Bool(..) => Type::Bool,
            Value::Tuple(..) => unimplemented!(),
            Value::Unit => unimplemented!(),
            Value::Err => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    I32,
    F64,
    Bool,
    Tuple(Vec<Type>),
}

impl Type {
    fn get_size(&self) -> usize {
        match self {
            Type::I32 | Type::F64 | Type::Bool => 1,
            Type::Tuple(parts) => parts.iter().map(|part| part.get_size()).sum(),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::F64 => write!(f, "F64"),
            Type::I32 => write!(f, "I32"),
            Type::Bool => write!(f, "Bool"),
            Type::Tuple(parts) => {
                write!(f, "Tuple(")?;
                for part in parts {
                    write!(f, "{}, ", part)?;
                }
                write!(f, ")")
            }
        }
    }
}
