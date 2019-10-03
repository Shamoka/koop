use crate::stage1;
use crate::AllocError;
use crate::memtree::{MemTree, MemTreeNode, TakeResult};
use crate::area::Area;
use crate::block::Block;
use crate::addr::Addr;

use core::mem::size_of;

const BUCKETS: usize = 49;

pub struct Allocator {
    internal: stage1::Allocator,
    buddies: [MemTree; BUCKETS],
    blocks: MemTree,
    mem_tree_node_order: usize,
    stock: *mut MemTreeNode,
    stock_count: usize
}

impl Allocator {
    pub fn new(stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
        let mut allocator = Allocator {
            internal: stage1,
            buddies: [MemTree::new(); BUCKETS],
            blocks: MemTree::new(),
            mem_tree_node_order: 0,
            stock: 0 as *mut MemTreeNode,
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
            let block = Block::new(ptr as usize, 0);
            match self.blocks.find(block.no_sign_addr()) {
                Some(node) => {
                    (*node).remove();
                    (*node).init(&(*node).content);
                    self.dealloc_recurse(node);
                },
                None => ()
            }
        }
    }
    
    pub fn inspect(&mut self) {
        self.blocks.inspect();
    }

    fn dealloc_recurse(&mut self, node: *mut MemTreeNode) {
        unsafe {
            match self.buddies[(*node).content.order].find((*node).content.buddy_addr()) {
                Some(buddy_node) => {
                    (*buddy_node).remove();
                    (*node).content.merge(&(*buddy_node).content);
                    (*buddy_node).left = self.stock;
                    self.stock = buddy_node;
                    self.stock_count += 1;
                    self.dealloc_recurse(node);
                },
                None => self.buddies[(*node).content.order].insert(node)
            }
        }
    }

    pub fn alloc(&mut self, len: usize) -> *mut u8 {
        if len == 0 {
            return 0 as *mut u8;
        }
        let target = self.get_order(len);
        match self.alloc_recurse(self.mem_tree_node_order, self.mem_tree_node_order) {
            Ok(node_block) => {
                match self.alloc_recurse(target, target) {
                    Ok(block) => {
                        unsafe {
                            (*(node_block.sign_addr() as *mut MemTreeNode)).init(&block);
                            self.blocks.insert(node_block.sign_addr() as *mut MemTreeNode);
                        }
                        block.sign_addr() as *mut u8
                    }
                    Err(_) => 0 as *mut u8
                }
            },
            Err(_) => 0 as *mut u8
        }
    }

    fn alloc_recurse(&mut self, order: usize, target: usize) -> Result<Block, AllocError> {
        if order >= BUCKETS {
            return Err(AllocError::OutOfMemory);
        }
        loop {
            match self.buddies[order].take() {
                TakeResult::Block(block) => {
                    if (block.order == target && target >= 12)
                        || (target < 12 && block.order == 12) {
                            if let Err(_) = self.internal.map(&Area::new(block.addr, block.size())) {
                                return self.alloc_recurse(order, target);
                        }
                    }
                    return Ok(block)
                },
                TakeResult::Node(node) => {
                    unsafe {
                        (*node).left = self.stock;
                        self.stock = node;
                        self.stock_count += 1;
                        return Ok((*node).content)
                    }
                },
                TakeResult::Empty => {
                    match self.alloc_recurse(order + 1, target) {
                        Ok(new_block) => return self.handle_new_block(new_block, order, target),
                        Err(_) => ()
                    }
                }
            }
        }
    }

    fn handle_new_block(&mut self, mut block: Block, order: usize, target: usize)
        -> Result<Block, AllocError> {
            match block.split() {
                Some(new_block) => {
                    if (block.order == target && target >= 12)
                        || (target < 12 && block.order == 12) {
                            if let Err(_) = self.internal.map(&Area::new(block.addr, block.size())) {
                                if let Err(_) = self.internal.map(&Area::new(new_block.addr, new_block.size())) {
                                    return Err(AllocError::InUse);
                                }
                                return Ok(new_block);
                            }
                            if let Err(error) = self.insert_block_discard(order, &new_block) {
                                return Err(error);
                            }
                            return Ok(block);
                        }
                    if let Err(error) = self.insert_block_discard(order, &block) {
                        return Err(error);
                    }
                    return Ok(new_block);
                },
                None => return Err(AllocError::OutOfMemory)
            }
        }

    fn insert_block_discard(&mut self, order: usize, block: &Block) -> Result<(), AllocError> {
        match self.buddies[order].insert_block(block) {
            true => Ok(()),
            false => {
                match self.stock.is_null() {
                    false => {
                        let node = self.stock;
                        unsafe {
                            self.stock = (*node).left;
                            (*node).init(block);
                        }
                        self.stock_count -= 1;
                        self.buddies[order].insert(node);
                        Ok(())
                    },
                    true => {
                        match self.alloc_recurse(self.mem_tree_node_order, self.mem_tree_node_order) {
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
