use crate::stage1;
use crate::AllocError;
use crate::area::Area;
use crate::block::Block;
use crate::memtree;
use crate::slab::Slab;

use core::alloc::Layout;

const BUCKETS: usize = 49;

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
        unsafe {
            if let Ok(slab_block) = allocator.alloc_iter(21, 1 << 21) {
                allocator.node_slab.init(&slab_block);
            } else {
                panic!("Cannot create node slab");
            }
        }
        allocator
    }

    pub fn inspect(&self) {
        self.internal.frame_allocator.inspect();
        self.blocks.inspect();
    }

    pub fn dealloc(&mut self, ptr: *mut u8) {
        let mut block = Block::new(ptr as usize, 0);
        block.remove_sign();
        match self.blocks.delete(block.addr) {
            Some(node) => {
                unsafe {
                    block.order = (*node).content.order;
                    if self.node_slab.give(node) == false {
                        (*(node as *mut Block)).addr = node as usize;
                        (*(node as *mut Block)).order = self.node_slab.order;
                        self.dealloc_recurse(*(node as *mut Block));
                    }
                }
                self.dealloc_recurse(block);
            },
            None => loop {}
        }
    }

    fn dealloc_frame(&mut self, mut block: Block) {
        block.add_sign();
        let area = Area::new(block.addr, block.size());
        for addr in area.pages() {
            match self.internal.unmap(&addr) {
                Ok(frame) => {
                    if let false = self.internal.frame_allocator.dealloc(frame) {
                        if let Ok(frame_block) = self.alloc_iter(12, 1 << 12) {
                            self.internal.frame_allocator.pool(&frame_block);
                        } else {
                            panic!("Cannot pool frame nodes");
                        }
                    }
                },
                Err(error) => panic!("Invalid unmap {} {} {} {:?}",
                    block.addr, block.size(), addr.addr, error)
            }
        }
    }

    fn dealloc_recurse(&mut self, mut block: Block) {
        unsafe {
            if block.order == 12 {
                self.dealloc_frame(block);
            }
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
                        (*(buddy_node as *mut Block)).addr = buddy_node as usize;
                        (*(buddy_node as *mut Block)).order = self.node_slab.order;
                        self.dealloc_recurse(*(buddy_node as *mut Block));
                    }
                    self.dealloc_recurse(block);
                },
                None => {
                    if let Ok(mut new_node) = self.alloc_node() {
                        (*new_node).content = block;
                        self.buddies[block.order].insert(new_node);
                    }
                }
            }
        }
    }

    pub fn alloc(&mut self, layout: &Layout) -> *mut u8 {
        if layout.size() == 0 {
            return 0 as *mut u8;
        }
        let target = self.get_order(layout.size());
        match self.alloc_node() {
            Ok(new_node) => {
                match self.alloc_iter(target, layout.align()) {
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
                match self.alloc_iter(self.node_slab.order, 1 << self.node_slab.order) {
                    Ok(mut block) => {
                        block.add_sign();
                        Ok(block.addr as *mut memtree::Node<'a>)
                    },
                    Err(error) => Err(error)
                }
            }
        }
    }

    fn alloc_iter(&mut self, target: usize, align: usize) -> Result<Block, AllocError> {
        let mut order = target;
        while let Some(mut block) = self.choose_block(order, align) {
            while block.order > target {
                if block.should_map(block.order, target) {
                    match self.internal.map(&block) {
                        Err(AllocError::InUse) => break,
                        Err(error) => return Err(error),
                        Ok(_) => {}
                    }
                }
                if let Some(new_block) = block.split(align) {
                    if self.buddies[new_block.order].insert_block(&new_block) == false {
                        unsafe {
                            match self.alloc_node() {
                                Ok(node) => {
                                    (*node).content = new_block;
                                    self.buddies[new_block.order].insert(node);
                                },
                                Err(_) => panic!("Cannot create new block")
                            };
                        }
                    }
                }
            }
            if block.order == target {
                if block.should_map(order, target) {
                    match self.internal.map(&block) {
                        Err(AllocError::InUse) => {
                            order = block.order;
                            continue
                        },
                        Err(error) => return Err(error),
                        Ok(_) => {}
                    }
                }
                return Ok(block);
            }
            order = block.order;
        }
        Err(AllocError::OutOfMemory)
    }

    fn choose_block(&mut self, order: usize, align: usize) -> Option<Block> {
        unsafe {
            for i in order..BUCKETS {
                match self.buddies[i].take(align) {
                    memtree::TakeResult::Block(block_taken) => return Some(block_taken),
                    memtree::TakeResult::Node(node) => {
                        let block = (*node).content;
                        if self.node_slab.give(node) == false {
                            (*(node as *mut Block)).addr = node as usize;
                            (*(node as *mut Block)).order = self.node_slab.order;
                            self.dealloc_recurse(*(node as *mut Block));
                        }
                        return Some(block);
                    }
                    memtree::TakeResult::Empty => {}
                }
            }
            None
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
