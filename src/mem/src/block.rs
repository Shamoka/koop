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

    pub fn split(&mut self, area: &Area) -> Option<Block> {
        if self.order == 0 {
            return None;
        }
        self.order -= 1;
        let new_block_size = self.size();
        if area.base.addr < self.addr + new_block_size {
            let ret = Some(Block {
                order: self.order,
                addr: self.addr
            });
            self.addr += new_block_size;
            ret
        } else {
            Some(Block {
                order: self.order,
                addr: self.addr + new_block_size
            })
        }
    }

    pub fn contains(&self, area: &Area) -> bool {
        self.addr <= area.base.no_sign()
            && self.size() >= area.len + area.base.no_sign() - self.addr
    }
}
