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
    pub fn new(mut stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
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
            if self.alloc_nodes(BUCKETS - order + 1) == false {
                return 0 as *mut u8;
            }
        }
        0 as *mut u8
    }

    fn alloc_nodes(&mut self, nb_nodes: usize) -> bool {
        while self.stock_count < nb_nodes {
            if self.alloc_nodes_recurse(self.mem_tree_node_order) == false {
                return false;
            }
        }
        true
    }

    fn alloc_nodes_recurse(&mut self, order: usize) -> bool {
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
                    Some(value) => {
                        if order == 12 {
                            self.internal.map(&Area::new(value.addr, value.size()));
                        }
                        if order > self.mem_tree_node_order {
                            self.buddies[order - 1].root = Some(MemTreeNode::new(&value));
                        } else {
                            let new_node = value.addr as *mut MemTreeNode;
                            unsafe {
                                (*new_node).left = self.stock;
                            }
                            self.stock = Some(new_node);
                            self.stock_count += 1;
                        }
                        return true;
                    },
                    None => {
                        if self.alloc_nodes_recurse(order + 1) {
                            return self.alloc_nodes_recurse(order);
                        }
                    }
                }
            }
        }
        false
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
