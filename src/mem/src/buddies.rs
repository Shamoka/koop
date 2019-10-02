use crate::memtree::{MemTree, MemTreeNode};
use crate::block::Block;
use crate::area::Area;
use crate::pool::Pool;

pub struct Buddies {
    content: [Option<MemTree>; 64],
    pool: *mut Pool<MemTreeNode>
}

impl Buddies {
    pub fn new(node_pool: *mut Pool<MemTreeNode>) -> Buddies {
        let mut buddies = Buddies {
            content: [None; 64],
            pool: node_pool
        };
        buddies.content[63] = Some(MemTree::new(buddies.pool));
        if let Some(ref mut tree) = buddies.content[63] {
            tree.insert(&Block::new(0, 63));
        };
        buddies
    }

    pub fn get_block(&mut self, area: &Area) -> Option<Block> {
        let order = self.find_order(area);
        for i in order..64 {
            if let Some(node) = self.get_block_in_order(area, i) {
                let mut block = unsafe { (*node).content };
                unsafe { (*node).remove(); }
                while let Some(unwanted_block) = block.split(area) {
                    if let None = self.content[unwanted_block.order] {
                        self.content[unwanted_block.order] = Some(MemTree::new(self.pool));
                    }
                    self.content[unwanted_block.order].unwrap().insert(&unwanted_block);
                }
                return Some(block);
            }
        }
        None
    }

    fn get_block_in_order(&self, area: &Area, order: usize) -> Option<*mut MemTreeNode> {
        match self.content[order] {
            Some(mut tree) => tree.find(area),
            None => None
        }
    }

    fn find_order(&self, area: &Area) -> usize {
        for i in 0..64 {
            if 1 << i > area.len {
                return i;
            }
        }
        return 63;
    }
}
