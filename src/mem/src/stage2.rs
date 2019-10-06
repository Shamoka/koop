use crate::stage1;
use crate::AllocError;
use crate::area::Area;
use crate::block::Block;
use crate::memtree;
use crate::slab::Slab;

const BUCKETS: usize = 48;

pub struct Allocator<'a> {
    internal: stage1::Allocator,
    buddies: [memtree::Tree<'a>; BUCKETS],
    blocks: memtree::Tree<'a>,
    node_slab: Slab<memtree::Node<'a>>
}

impl<'a> Allocator<'a> {
    pub fn new(mb2: multiboot2::Info) -> Allocator<'a> {
        let mut allocator = Allocator {
            internal: stage1::Allocator::new(mb2),
            buddies: [memtree::Tree::new(); BUCKETS],
            blocks: memtree::Tree::new(),
            node_slab: Slab::new()
        };
        allocator.buddies[BUCKETS - 1].insert_block(&Block::new(0, BUCKETS - 1));
        if let Ok(slab_block) = allocator.alloc_recurse(21, 21) {
            unsafe {
                allocator.node_slab.init(&slab_block);
            }
        } else {
            panic!("Cannot create node slab");
        }
        allocator
    }

    pub fn inspect(&self) {
        self.blocks.inspect();
    }

    pub fn dealloc(&mut self, ptr: *mut u8) {
        let mut block = Block::new(ptr as usize, 0);
        block.remove_sign();
        match self.blocks.delete(block.addr) {
            Some(node) => {
                unsafe {
                    if self.node_slab.give(node) == false {
                        self.dealloc_recurse(&mut Block::new(node as usize, self.node_slab.order));
                    }
                }
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
                    if self.node_slab.give(buddy_node) == false {
                        self.dealloc_recurse(&mut Block::new(buddy_node as usize, self.node_slab.order));
                    }
                    self.dealloc_recurse(block);
                },
                None => {
                    match self.alloc_node() {
                        Ok(mut new_node) => {
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
        match self.alloc_node() {
            Ok(new_node) => {
                match self.alloc_recurse(target, target) {
                    Ok(mut block) => {
                        block.remove_sign();
                        unsafe {
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

    fn alloc_node(&mut self) -> Result<*mut memtree::Node<'a>, AllocError> {
        unsafe {
            if let Some(node) = self.node_slab.get() {
                Ok(node)
            } else {
                match self.alloc_recurse(self.node_slab.order, self.node_slab.order) {
                    Ok(mut block) => {
                        block.add_sign();
                        Ok(block.addr as *mut memtree::Node<'a>)
                    },
                    Err(error) => Err(error)
                }
            }
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
                        let block = (*node).content;
                        if (block.order == target && target >= 12)
                            || (target < 12 && block.order == 12) {
                                if let Err(_) = self.internal.map(&Area::new(block.addr, block.size())) {
                                    return self.alloc_recurse(order, target);
                                }
                            }
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
