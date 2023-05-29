use std::mem::size_of;

use crate::utils::Mem;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Value {
    def: TypeDef,
    mem: Mem,
}

impl Value {
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

#[derive(Debug)]
pub struct ValueRef {
    def: TypeDef,
    reg: usize,
}
