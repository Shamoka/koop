pub struct Bits {
    pub value: usize
}

impl Bits {
    pub const fn new(value: usize) -> Bits {
        Bits {
            value: value
        }
    }

    pub fn get_mask(begin: usize, end: usize) -> usize {
        let mut mask = 0;
        for i in begin..=end {
            mask = (mask << 1) + 1;
        }
        mask << begin
    }

    pub fn get_bits(&self, begin: usize, end: usize) -> usize {
        self.value & Bits::get_mask(begin, end)
    }

    pub fn set_bits(&mut self, begin: usize, end: usize, value: usize) {
        let mask = value << begin;
        self.value |= mask;
    }
}
