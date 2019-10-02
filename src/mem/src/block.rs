use crate::area::Area;

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
