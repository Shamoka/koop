use crate::stage1;
use crate::AllocError;
use crate::memtree::{MemTree, MemTreeNode};
use crate::area::Area;
use crate::block::Block;

use core::mem::size_of;

const BUCKETS: usize = 49;

pub struct Allocator {
    internal: stage1::Allocator,
    buddies: [MemTree; BUCKETS],
    mem_tree_node_order: usize,
    stock: Option<*mut MemTreeNode>,
    stock_count: usize
}

impl Allocator {
    pub fn new(stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
        let mut allocator = Allocator {
            internal: stage1,
            buddies: [MemTree::new(); BUCKETS],
            mem_tree_node_order: 0,
            stock: None,
            stock_count: 0
        };
        while 1 << allocator.mem_tree_node_order < size_of::<MemTreeNode>() {
            allocator.mem_tree_node_order += 1;
        }
        allocator.buddies[BUCKETS - 1] = MemTree::new_init(&Block::new(0, BUCKETS - 1));
        Ok(allocator)
    }

    pub fn alloc(&mut self, len: usize) -> *mut u8 {
        if len > 1 << BUCKETS {
            return 0 as *mut u8;
        }
        let order = self.get_order(len);
        if BUCKETS - order > self.stock_count {
            if let Err(_) = self.alloc_nodes(BUCKETS - order + 1) {
                return 0 as *mut u8;
            }
        }
        match self.alloc_recurse(order) {
            Ok(block) => block.addr as *mut u8,
            Err(_) => 0 as *mut u8
        }
    }

    fn alloc_nodes(&mut self, nb_nodes: usize) -> Result<(), AllocError> {
        while self.stock_count < nb_nodes {
            match self.alloc_recurse(self.mem_tree_node_order)  {
                Ok(block) => {
                    let node = block.addr as *mut MemTreeNode;
                    unsafe {
                        (*node).left = self.stock;
                    }
                    self.stock = Some(node);
                    self.stock_count += 1;
                },
                Err(error) => return Err(error)
            };
        }
        Ok(())
    }

    fn alloc_recurse(&mut self, order: usize) -> Result<Block, AllocError> {
        if order >= BUCKETS {
            return Err(AllocError::OutOfMemory);
        }
        match self.buddies[order].take() {
            (block, node) => {
                if let Some(value) = node {
                    unsafe {
                        (*value).left = self.stock.take();
                    }
                    self.stock = Some(value);
                    self.stock_count += 1;
                }
                match block {
                    Some(value) => self.handle_new_block(value, order),
                    None => {
                        match self.alloc_recurse(order + 1) {
                            Ok(new_block) => self.handle_new_block(new_block, order),
                            Err(error) => return Err(error)
                        }
                    }
                }
            }
        }
    }

    fn handle_new_block(&mut self, mut block: Block, order: usize) -> Result<Block, AllocError> {
        match block.split() {
            Some(new_block) => {
                self.buddies[order - 1].root = Some(MemTreeNode::new(&new_block));
                if order == 12 {
                    if let Err(_) = self.internal.map(&Area::new(block.addr, block.size())) {
                        return self.alloc_recurse(order + 1);
                    }
                }
                return Ok(block);
            },
            None => return Err(AllocError::OutOfMemory)
        }
    }

    fn get_order(&self, len: usize) -> usize {
        for i in 0..BUCKETS {
            if 1 << i > len {
                return i;
            }
        }
        0
    }
}
