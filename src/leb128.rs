pub trait Leb128 {
    fn write_leb128(self, b: &mut Vec<u8>) -> ();
}

const CONTINUATION_BIT: u8 = 1 << 7;

impl Leb128 for usize {
    fn write_leb128(mut self, b: &mut Vec<u8>) {
        loop {
            let mut byte: u8 = (self & 0b1111111) as u8;
            self >>= 7;

            if self != 0 {
                byte |= CONTINUATION_BIT;
            }

            b.push(byte);

            if self == 0 {
                break;
            }
        }
    }
}

impl Leb128 for i32 {
    fn write_leb128(mut self, b: &mut Vec<u8>) {
        loop {
            let mut byte: u8 = self as u8;
            self >>= 6;

            let done = self == 0 || self == -1;

            if done {
                byte &= !CONTINUATION_BIT;
            } else {
                self >>= 1;
                byte |= !CONTINUATION_BIT;
            }

            b.push(byte);

            if done {
                break;
            }
        }
    }
}
