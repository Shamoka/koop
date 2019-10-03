use crate::stage1;
use crate::AllocError;
use crate::memtree::{MemTree, MemTreeNode, TakeResult};
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
        allocator.buddies[BUCKETS - 1].insert_block(&Block::new(0, BUCKETS - 1));
        Ok(allocator)
    }

    pub fn alloc(&mut self, len: usize) -> *mut u8 {
        match self.alloc_recurse(self.get_order(len)) {
            Ok(block) => block.addr as *mut u8,
            Err(_) => 0 as *mut u8
        }
    }

    fn alloc_recurse(&mut self, order: usize) -> Result<Block, AllocError> {
        if order >= BUCKETS {
            return Err(AllocError::OutOfMemory);
        }
        match self.buddies[order].take() {
            TakeResult::Block(block) => {
                return self.handle_new_block(block, order);
            },
            TakeResult::Node(node) => {
                unsafe {
                    (*node).left = self.stock;
                    self.stock = Some(node);
                    Ok((*node).content)
                }
            },
            TakeResult::Empty => {
                match self.alloc_recurse(order + 1) {
                    Ok(new_block) => self.handle_new_block(new_block, order),
                    Err(error) => Err(error)
                }
            }
        }
    }

    fn handle_new_block(&mut self, mut block: Block, order: usize)
        -> Result<Block, AllocError> {
            let block_keep: &Block;
            let block_discard: &Block;
            match block.split() {
                Some(new_block) => {
                    block_keep = &block;
                    block_discard = &new_block;
                    self.buddies[order - 1].insert_block(block_discard);
                    if order == 12 {
                        if let Err(_) = self.internal.map(&Area::new(block_keep.addr, block.size())) {
                            return self.alloc_recurse(order + 1);
                        }
                    }
                    return Ok(*block_keep);
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
