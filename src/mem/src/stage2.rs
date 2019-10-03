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
    blocks: MemTree,
    mem_tree_node_order: usize,
    stock: Option<*mut MemTreeNode>,
    stock_count: usize
}

impl Allocator {
    pub fn new(stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
        let mut allocator = Allocator {
            internal: stage1,
            buddies: [MemTree::new(); BUCKETS],
            blocks: MemTree::new(),
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

    pub fn dealloc(&mut self, ptr: *mut u8) {
        unsafe {
            match self.blocks.find(ptr as usize) {
                Some(node) => {
                    (*node).remove();
                    self.dealloc_recurse(node);
                },
                None => ()
            }
        }
    }

    fn dealloc_recurse(&mut self, node: *mut MemTreeNode) {
        unsafe {
            match self.buddies[(*node).content.order].find((*node).content.buddy_addr()) {
                Some(buddy_node) => {
                    (*buddy_node).remove();
                    (*node).content.merge(&(*buddy_node).content);
                    (*buddy_node).left = self.stock;
                    self.stock = Some(buddy_node);
                    self.stock_count += 1;
                    self.dealloc_recurse(node);
                },
                None => self.buddies[(*node).content.order].insert(node)
            }
        }
    }

    pub fn alloc(&mut self, len: usize) -> *mut u8 {
        match self.alloc_recurse(self.mem_tree_node_order) {
            Ok(node_block) => {
                match self.alloc_recurse(self.get_order(len)) {
                    Ok(block) => {
                        unsafe {
                            (*(node_block.addr as *mut MemTreeNode)).init(&block);
                            self.blocks.insert(node_block.addr as *mut MemTreeNode);
                        }
                        block.addr as *mut u8
                    }
                    Err(_) => 0 as *mut u8
                }
            },
            Err(_) => 0 as *mut u8
        }
    }

    fn alloc_recurse(&mut self, order: usize) -> Result<Block, AllocError> {
        if order >= BUCKETS {
            return Err(AllocError::OutOfMemory);
        }
        match self.buddies[order].take() {
            TakeResult::Block(block) => {
                Ok(block)
            },
            TakeResult::Node(node) => {
                unsafe {
                    (*node).left = self.stock;
                    self.stock = Some(node);
                    self.stock_count += 1;
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
                    if let Err(error) = self.insert_block_discard(order, block_discard) {
                        return Err(error);
                    }
                    if order == 12 {
                        if let Err(_) = self.internal.map(&Area::new(block_keep.addr, block_keep.size())) {
                            return self.alloc_recurse(order);
                        }
                    }
                    return Ok(*block_keep);
                },
                None => return Err(AllocError::OutOfMemory)
            }
        }

    fn insert_block_discard(&mut self, order: usize, block: &Block) -> Result<(), AllocError> {
        match self.buddies[order].insert_block(block) {
            true => Ok(()),
            false => {
                match self.stock {
                    Some(node) => {
                        unsafe {
                            self.stock = (*node).left;
                            (*node).init(block);
                        }
                        self.stock_count -= 1;
                        self.buddies[order].insert(node);
                        Ok(())
                    },
                    None => {
                        match self.alloc_recurse(self.mem_tree_node_order) {
                            Ok(node_block) => {
                                let new_node = node_block.addr as *mut MemTreeNode;
                                unsafe {
                                    (*new_node).init(block);
                                }
                                self.buddies[order].insert(new_node);
                                Ok(())
                            },
                            Err(error) => Err(error)
                        }
                    }
                }
            }
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
