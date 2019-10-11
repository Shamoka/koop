use crate::frame::Frame;
use crate::block::Block;

use core::mem::size_of;

pub struct Stack {
    root: *mut Node,
    pool: *mut Node
}

struct Node {
    pub frame: Frame,
    pub next: *mut Node
}

impl Node {
    pub unsafe fn count(&self) -> usize {
        match self.next.is_null() {
            true => 1,
            false => 1 + (*self.next).count()
        }
    }
}

impl Stack {
    pub const fn new() -> Stack {
        Stack {
            root: 0 as *mut Node,
            pool: 0 as *mut Node
        }
    }

    pub fn push(&mut self, frame: Frame) -> bool {
        unsafe {
            match self.pool.is_null() {
                true => false,
                false => {
                    let tmp = (*self.pool).next;
                    (*self.pool).frame = frame;
                    (*self.pool).next = self.root;
                    self.root = self.pool;
                    self.pool = tmp;
                    true
                }
            }
        }
    }

    pub fn pop(&mut self) -> Option<Frame> {
        unsafe {
            match self.root.is_null() {
                true => None,
                false => {
                    let tmp = (*self.root).next;
                    (*self.root).next = self.pool;
                    self.pool = self.root;
                    self.root = tmp;
                    Some((*self.pool).frame)
                }
            }
        }
    }

    pub fn pool(&mut self, block: &Block) {
        let count = block.size() / size_of::<Node>();
        let addr = block.addr as *mut Node;

        unsafe {
            for i in 0..count {
                let node = addr.offset(i as isize);
                (*node).next = self.pool;
                self.pool = node;
            }
        }
    }
}
