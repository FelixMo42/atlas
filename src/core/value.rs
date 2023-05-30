use std::{fmt::Display, mem::size_of};

use crate::utils::Mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDef {
    Unit,
    Bool,
    I32,
    F64,
}

impl TypeDef {
    pub fn size(&self) -> usize {
        match self {
            TypeDef::Unit => 0,
            TypeDef::Bool => size_of::<bool>(),
            TypeDef::I32 => size_of::<i32>(),
            TypeDef::F64 => size_of::<f64>(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    def: TypeDef,
    mem: Mem,
}

impl Value {
    pub fn new(def: TypeDef, mem: Mem) -> Value {
        return Value { def, mem };
    }

    pub fn get_size(&self) -> usize {
        return self.get_type().size();
    }

    pub fn get_type(&self) -> TypeDef {
        return self.def;
    }

    pub fn get_bytes(&self) -> &Mem {
        return &self.mem;
    }
}

impl Value {
    pub fn as_i32(&self) -> i32 {
        if self.def != TypeDef::I32 {
            panic!("No an int!");
        }

        i32::from_be_bytes(self.mem.get_slice(0))
    }

    pub fn as_f64(&self) -> f64 {
        if self.def != TypeDef::F64 {
            panic!("No a float!");
        }

        f64::from_be_bytes(self.mem.get_slice(0))
    }

    pub fn as_bool(&self) -> bool {
        if self.def != TypeDef::Bool {
            panic!("No a bool!");
        }

        return self.mem.bytes[0] == 1;
    }
}

impl Value {
    pub fn i32(value: i32) -> Value {
        Value {
            def: TypeDef::I32,
            mem: Mem::new(value.to_be_bytes().to_vec()),
        }
    }

    pub fn f64(value: f64) -> Value {
        Value {
            def: TypeDef::F64,
            mem: Mem::new(value.to_be_bytes().to_vec()),
        }
    }

    pub fn bool(value: bool) -> Value {
        Value {
            def: TypeDef::Bool,
            mem: Mem::new(if value { vec![1] } else { vec![0] }),
        }
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDef::I32 => write!(f, "I32"),
            TypeDef::F64 => write!(f, "F64"),
            TypeDef::Bool => write!(f, "Bool"),
            _ => unimplemented!(),
        }
    }
}
