use crate::core::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mem {
    pub bytes: Vec<u8>,
}

impl Mem {
    pub fn new(bytes: Vec<u8>) -> Mem {
        return Mem { bytes };
    }

    pub fn of_size(initial_size: usize) -> Mem {
        Mem::new(vec![0; initial_size])
    }

    pub fn default() -> Mem {
        Mem::of_size(1024)
    }
}

impl Mem {
    pub fn get(&self, reg: usize, def: TypeDef) -> Value {
        let bytes = Mem {
            bytes: self.bytes[reg..reg + def.size()].iter().cloned().collect(),
        };

        return Value::new(def, bytes);
    }

    pub fn set(&mut self, reg: usize, mem: &Mem) {
        for i in 0..mem.bytes.len() {
            self.bytes[reg + i] = mem.bytes[i];
        }
    }

    pub fn get_slice<const N: usize>(&self, reg: usize) -> [u8; N] {
        self.bytes[reg..reg + N]
            .try_into()
            .expect("failed to read slice from array")
    }
}
