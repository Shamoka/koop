use crate::bits::Bits;

const PAGE_OFFSET_LEN: usize = 12;
const TABLE_INDEX_LEN: usize = 9;

#[derive(Copy, Clone)]
pub struct Addr {
    pub bits: Bits
}

impl Addr {
    pub const fn new(value: usize) -> Addr {
        Addr {
            bits: Bits::new(value)
        }
    }

    pub fn get_table_index(&self, level: usize) -> usize {
        self.bits.get_bits(TABLE_INDEX_LEN * (level - 1) + PAGE_OFFSET_LEN,
                           TABLE_INDEX_LEN * level + PAGE_OFFSET_LEN - 1)
            >> (TABLE_INDEX_LEN * (level - 1) + PAGE_OFFSET_LEN)
    }

    pub fn get_table_addr(&self, level: usize) -> Addr {
        let mut addr = self.bits.value;
        for _ in 0..level {
            addr >>= 9;
            addr &= !0xfff;
            addr |= 0o177777_000_000_000_000_0000;
        }
        if addr & (1 << 47) == 0 {
            addr &= 0o000000_777_777_777_777_0000;
        }
        Addr::new(addr)
    }
}
