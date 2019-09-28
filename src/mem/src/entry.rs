use crate::bits::Bits;

pub const FLAG_PRESENT: usize = 1 << 0;
pub const FLAG_WRITABLE: usize = 1 << 1;

const ALL_FLAGS: usize = FLAG_PRESENT | FLAG_WRITABLE;

pub struct Entry {
    value: Bits
}

impl Entry {
    pub fn new(addr: usize, flags: usize) -> Entry {
        let mut entry = Entry { value: Bits::new(0) };
        entry.set_addr(addr);
        entry.set_flags(flags);
        entry
    }

    pub fn from_value(entry: usize) -> Entry {
        Entry {
            value: Bits::new(entry)
        }
    }

    pub fn value(&self) -> usize {
        self.value.value
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.value.set_bits(12, 51, addr >> 12);
    }

    pub fn set_flags(&mut self, flags: usize) {
        self.value.set_bits(0, 63, flags & ALL_FLAGS);
    }
}
