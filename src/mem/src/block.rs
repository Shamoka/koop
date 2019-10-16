#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Block {
    pub order: usize,
    pub addr: usize,
}

impl Block {
    pub const fn new(addr: usize, order: usize) -> Block {
        Block {
            order: order,
            addr: addr,
        }
    }

    pub fn satisfy_align(&self, align: usize) -> bool {
        self.addr & (align - 1) == 0
    }

    pub fn should_map(&self, order: usize, target: usize) -> bool {
        if self.addr + self.size() >= 0o776_000_000_000_0000 {
            return false;
        }
        if (order == target && target >= 12) || (target < 12 && order == 12) {
            return true;
        }
        false
    }

    pub fn add_sign(&mut self) {
        if self.addr & (1 << 47) == 0 {
            self.addr &= 0o000000_777_777_777_777_7777;
        } else {
            self.addr |= 0o177777_000_000_000_000_0000;
        }
    }

    pub fn remove_sign(&mut self) {
        self.addr &= 0o000000_777_777_777_777_7777;
    }

    pub const fn size(&self) -> usize {
        1 << self.order
    }

    pub fn buddy_addr(&self) -> usize {
        self.addr ^ self.size()
    }

    pub fn merge(&mut self, other: &Block) {
        if self.addr ^ self.size() != other.addr {
            panic!("Merging non buddy blocks {:?} {:?}", self, other);
        }
        self.order += 1;
        if self.addr > other.addr {
            self.addr = other.addr;
        }
    }

    pub fn split(&mut self, align: usize) -> Option<Block> {
        if self.order == 0 {
            return None;
        }
        self.order -= 1;
        if self.satisfy_align(align) {
            Some(Block {
                order: self.order,
                addr: self.buddy_addr(),
            })
        } else {
            let block = Block {
                order: self.order,
                addr: self.addr,
            };
            self.addr = self.buddy_addr();
            Some(block)
        }
    }
}
