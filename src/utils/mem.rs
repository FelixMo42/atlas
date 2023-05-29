#[derive(Clone, Debug)]
pub struct Mem {
    bytes: Vec<u8>,
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
    pub fn get(&self, reg: usize, size: usize) -> Mem {
        Mem {
            bytes: self.bytes[reg..reg + size].iter().cloned().collect(),
        }
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

    pub fn get_u32(&self, reg: usize) -> u32 {
        u32::from_be_bytes(self.get_slice(reg))
    }

    // pub fn set_u32(&mut self, reg: usize, value: u32) {
    //     let new = u32::to_be_bytes(value);

    //     self.bytes[reg + 0] = new[0];
    //     self.bytes[reg + 1] = new[1];
    //     self.bytes[reg + 2] = new[2];
    //     self.bytes[reg + 3] = new[3];
    // }
}
