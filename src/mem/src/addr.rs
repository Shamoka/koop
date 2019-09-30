use core::cmp::PartialEq;

#[derive(Debug, Copy, Clone)]
pub enum AddrType {
    Physical,
    Virtual
}

#[derive(Debug, Copy, Clone)]
pub struct Addr {
    pub addr: usize,
    pub addr_type: AddrType
}

impl Addr {
    pub const fn new(value: usize, addr_type: AddrType) -> Addr {
        Addr {
            addr: value,
            addr_type: addr_type
        }
    }

    pub fn get_table_index(&self, level: usize) -> usize {
        (self.addr & (0o777 << (12 + 9 * (level - 1)))) >> (12 + 9 * (level - 1))
    }

    pub fn get_table_addr(&self, level: usize, base: usize) -> Addr {
        let mut addr = self.addr & 0o000000_777_777_777_777_0000;
        for _ in 0..level {
            addr |= base << 48;
            addr >>= 9;
            addr &= !0xfff;
        }
        if addr & (1 << 47) == 0 {
            addr &= 0o000000_777_777_777_777_0000;
        } else {
            addr |= 0o177777_000_000_000_000_0000;
        }
        Addr::new(addr, AddrType::Virtual)
    }

    pub fn is_valid(&self) -> bool {
        match self.addr_type {
            AddrType::Virtual => {
                if self.addr & (1 << 47) == 0 {
                    self.addr & 0o177777_000_000_000_000_0000 == 0
                } else {
                    self.addr & 0o177777_000_000_000_000_0000 == 0o177777_000_000_000_000_0000
                }
            },
            AddrType::Physical => false
        }
    }

    pub fn to_valid(&mut self) {
        match self.addr_type {
            AddrType::Virtual => {
                if self.addr & (1 << 47) == 0 {
                    self.addr &= 0o000000_777_777_777_777_7777;
                } else {
                    self.addr |= 0o177777_000_000_000_000_0000;
                }
            },
            AddrType::Physical => {}
        }
    }
}

impl PartialEq for Addr {
    fn eq(&self, other: &Addr) -> bool {
        self.addr == other.addr
    }
}
