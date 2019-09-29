pub const FLAG_PRESENT: usize = 1 << 0;
pub const FLAG_WRITABLE: usize = 1 << 1;

const FLAG_BITS: usize = FLAG_PRESENT | FLAG_WRITABLE;
const ADDR_BITS: usize = 0x000f_ffff_ffff_f000;

pub struct Entry {
    addr: usize,
    flags: usize
}

impl Entry {
    pub fn new(addr: usize, flags: usize) -> Entry {
        Entry {
            addr: addr & ADDR_BITS,
            flags: flags & FLAG_BITS
        }
    }

    pub fn from_entry(value: usize) -> Entry {
        Entry {
            addr: value & ADDR_BITS,
            flags: value & !ADDR_BITS
        }
    }

    pub fn unused(&self) -> bool {
        self.addr | self.flags == 0
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr & ADDR_BITS;
    }

    pub fn set_flags(&mut self, flags: usize) {
        self.flags |= flags & FLAG_BITS;
    }

    pub fn value(&self) -> usize {
        self.addr | self.flags
    }
}
