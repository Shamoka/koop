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
