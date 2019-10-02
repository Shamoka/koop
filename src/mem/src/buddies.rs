use crate::memtree::{MemTree, MemTreeNode};
use crate::block::Block;
use crate::area::Area;

const BUDDIES_BUCKETS: usize = 49;

pub struct Buddies {
    content: [Option<MemTree>; BUDDIES_BUCKETS],
}

impl Buddies {
    pub fn new() -> Buddies {
        Buddies {
            content: [None; BUDDIES_BUCKETS],
        }
    }

    pub fn get_block(&mut self, area: &Area) -> Option<Block> {
        let order = self.find_order(area);
        for i in order..BUDDIES_BUCKETS {
            /*
            if let Some(node) = self.get_block_in_order(area, i) {
                let mut block = unsafe { (*node).content };
                unsafe {
                    (*node).remove();
                }
                while let Some(unwanted_block) = block.split(area) {
                    if let None = self.content[unwanted_block.order] {
                        self.content[unwanted_block.order] = Some(MemTree::new());
                    }
                    self.content[unwanted_block.order].unwrap().insert(&unwanted_block);
                }
                return Some(block);
            }
            */
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
        for i in 0..BUDDIES_BUCKETS {
            if 1 << i > area.len {
                return i;
            }
        }
        return 63;
    }
}
