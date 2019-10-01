use crate::frame;

pub struct Block {
    pub order: usize,
    pub addr: usize,
}

pub struct BlockBuilder {
    pos: usize,
    end: usize
}

impl Block {
    pub fn new(addr: usize, order: usize) -> Block {
        Block {
            order: order,
            addr: addr
        }
    }

    pub fn size(&self) -> usize {
        frame::FRAME_SIZE << self.order
    }

    pub fn from_memory_bounds(start: usize, end: usize) -> BlockBuilder {
        BlockBuilder {
            pos: start,
            end: end
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

impl Iterator for BlockBuilder {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        let current_size = self.end - self.pos;
        if current_size < frame::FRAME_SIZE {
            return None
        }
        for order in 1..51 {
            if frame::FRAME_SIZE << (50 - order + 1) < current_size {
                let ret = Some(Block::new(self.pos, 50 - order + 1));
                self.pos += frame::FRAME_SIZE << (50 - order + 1);
                return ret;
            }
        }
        None
    }
}
