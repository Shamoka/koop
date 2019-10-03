#[derive(Copy, Clone)]
pub struct Block {
    pub order: usize,
    pub addr: usize,
}

impl Block {
    pub fn new(addr: usize, order: usize) -> Block {
        Block {
            order: order,
            addr: addr
        }
    }

    pub fn sign_addr(&self) -> usize {
        if self.addr & (1 << 47) == 0 {
            self.addr & 0o000000_777_777_777_777_7777
        } else {
            self.addr | 0o177777_000_000_000_000_0000
        }
    }

    pub fn no_sign_addr(&self) -> usize {
        self.addr & 0o000000_777_777_777_777_7777
    }

    pub fn size(&self) -> usize {
        1 << self.order
    }

    pub fn buddy_addr(&self) -> usize {
        self.addr ^ (1 << self.order)
    }

    pub fn merge(&mut self, other: &Block) {
        if self.buddy_addr() == other.addr {
            self.order += 1;
            if self.addr > other.addr {
                self.addr = other.addr;
            }
        }
    }

    pub fn split(&mut self) -> Option<Block> {
        if self.order == 0 {
            return None;
        }
        self.order -= 1;
        Some(Block {
            order: self.order,
            addr: self.addr + self.size()
        })
    }
}
