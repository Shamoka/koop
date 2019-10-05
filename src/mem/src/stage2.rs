use crate::stage1;
use crate::AllocError;
use crate::area::Area;
use crate::block::Block;
use crate::memtree;

use core::mem::size_of;

const BUCKETS: usize = 48;

pub struct Allocator<'a> {
    internal: stage1::Allocator,
    buddies: [memtree::Tree<'a>; BUCKETS],
    blocks: memtree::Tree<'a>,
    mem_tree_node_order: usize
}

impl<'a> Allocator<'a> {
    pub fn new(mb2: multiboot2::Info) -> Allocator<'a> {
        let mut allocator = Allocator {
            internal: stage1::Allocator::new(mb2),
            buddies: [memtree::Tree::new(); BUCKETS],
            blocks: memtree::Tree::new(),
            mem_tree_node_order: 0
        };
        while 1 << allocator.mem_tree_node_order < size_of::<memtree::Node>() {
            allocator.mem_tree_node_order += 1;
        }
        allocator.buddies[BUCKETS - 1].insert_block(&Block::new(0, BUCKETS - 1));
        allocator
    }

    pub fn inspect(&self) {
        self.blocks.inspect();
    }

    pub fn dealloc(&mut self, ptr: *mut u8) {
        let mut block = Block::new(ptr as usize, 0);
        block.remove_sign();
        match self.blocks.delete(block.addr) {
            Some(_node) => {
                // TODO: pool nodes
                self.dealloc_recurse(&mut block);
            },
            None => panic!("Block {:?} not found in dealloc", block)
        }
    }

    fn dealloc_recurse(&mut self, block: &mut Block) {
        unsafe {
            if let Some(buddies_block) = self.buddies[block.order].block {
                if block.buddy_addr() == buddies_block.addr {
                    block.merge(&buddies_block);
                    self.buddies[block.order - 1].block = None;
                    return self.dealloc_recurse(block);
                }
            }
            match self.buddies[block.order].delete(block.buddy_addr()) {
                Some(buddy_node) => {
                    block.merge(&(*buddy_node).content);
                    // TODO: pool nodes
                    self.dealloc_recurse(block);
                },
                None => {
                    match self.alloc_recurse(self.mem_tree_node_order, self.mem_tree_node_order) {
                        Ok(mut node_block) => {
                            node_block.add_sign();
                            let new_node: *mut memtree::Node<'a> = node_block.addr as *mut memtree::Node;
                            (*new_node).content = *block;
                            self.buddies[block.order].insert(new_node);
                        },
                        _ => ()
                    }
                }
            }
        }
    }

    pub fn alloc(&mut self, len: usize) -> *mut u8 {
        if len == 0 {
            return 0 as *mut u8;
        }
        let target = self.get_order(len);
        match self.alloc_recurse(self.mem_tree_node_order, self.mem_tree_node_order) {
            Ok(mut node_block) => {
                match self.alloc_recurse(target, target) {
                    Ok(mut block) => {
                        block.remove_sign();
                        node_block.add_sign();
                        unsafe {
                            let new_node: *mut memtree::Node<'a> = node_block.addr as *mut memtree::Node;
                            (*new_node).content = block;
                            self.blocks.insert(new_node)
                        }
                        block.add_sign();
                        block.addr as *mut u8
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
                memtree::TakeResult::Block(block) => {
                    if (block.order == target && target >= 12)
                        || (target < 12 && block.order == 12) {
                            if let Err(_) = self.internal.map(&Area::new(block.addr, block.size())) {
                                return self.alloc_recurse(order, target);
                            }
                        }
                    return Ok(block)
                },
                memtree::TakeResult::Node(node) => {
                    unsafe {
                        return Ok((*node).content)
                    }
                },
                memtree::TakeResult::Empty => {
                    match self.alloc_recurse(order + 1, target) {
                        Ok(mut new_block) => return self.handle_new_block(&mut new_block, order, target),
                        Err(AllocError::InUse) => {}
                        Err(error) => return Err(error)
                    };
                }
            }
        }
    }

    fn handle_new_block(&mut self, block: &mut Block, order: usize, target: usize)
        -> Result<Block, AllocError> {
            match block.split() {
                Some(new_block) => {
                    if (block.order == target && target >= 12)
                        || (target < 12 && block.order == 12) {
                            if let Err(_) = self.internal.map(&Area::new(new_block.addr, new_block.size())) {
                                if let Err(_) = self.internal.map(&Area::new(block.addr, block.size())) {
                                    return Err(AllocError::InUse);
                                }
                                return Ok(*block);
                            }
                        }
                    if self.buddies[order].insert_block(block) == false {
                        panic!("TODO: fix memory allocation");
                    }
                    return Ok(new_block);
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
