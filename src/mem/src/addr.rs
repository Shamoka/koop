use crate::bits::Bits;

const PAGE_OFFSET_LEN: usize = 12;
const TABLE_INDEX_LEN: usize = 9;

pub struct Addr {
    pub bits: Bits
}

pub struct AddrTablesIter<'a> {
    addr: &'a Addr,
    i: usize
}

impl Addr {
    pub const fn new(value: usize) -> Addr {
        Addr {
            bits: Bits::new(value)
        }
    }

    pub fn tables(&self) -> AddrTablesIter {
        AddrTablesIter {
            addr: self,
            i: 4
        }
    }

    pub fn table_offset(&self, table: usize) -> usize {
        self.bits.get_bits(PAGE_OFFSET_LEN + (4 - table) * TABLE_INDEX_LEN,
                           PAGE_OFFSET_LEN + (5 - table) * TABLE_INDEX_LEN - 1)
            >> (PAGE_OFFSET_LEN + (4 - table) * TABLE_INDEX_LEN)
    }
}

impl<'a> Iterator for AddrTablesIter<'a> {
    type Item = Addr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == 0 {
            return None;
        }
        let mut addr = Addr::new(0);
        addr.bits.set_bits(0, PAGE_OFFSET_LEN - 1, self.addr.table_offset(5 - self.i));
        for i in 0..4 {
            if i < self.i {
                addr.bits.set_bits(PAGE_OFFSET_LEN + TABLE_INDEX_LEN * (3 - i),
                                   PAGE_OFFSET_LEN + TABLE_INDEX_LEN * (4 - i),
                                   0o777);
            } else {
                addr.bits.set_bits(PAGE_OFFSET_LEN + TABLE_INDEX_LEN * (3 - i),
                                   PAGE_OFFSET_LEN + TABLE_INDEX_LEN * (4 - i),
                                   self.addr.table_offset(4 - i));
            }
        }
        if addr.bits.get_bits(47, 47) != 0 {
            addr.bits.set_bits(48, 64, !0);
        }
        self.i -= 1;
        Some(addr)
    }
}
