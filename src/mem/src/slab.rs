use crate::block::Block;

use core::mem::size_of;

const NODE_AVAILABLE_BIT: usize = 1 << 63;

struct Node<T: Sized> {
    content: T,
    count: usize,
}

#[derive(Copy, Clone)]
pub struct Slab<T: Sized> {
    base: *mut Node<T>,
    end: *mut Node<T>,
    cap: usize,
    pub order: usize,
}

impl<T: Sized> Slab<T> {
    pub const fn new() -> Slab<T> {
        Slab {
            base: 0 as *mut Node<T>,
            end: 0 as *mut Node<T>,
            cap: 0,
            order: 0,
        }
    }

    pub unsafe fn init(&mut self, block: &Block) {
        self.base = block.addr as *mut Node<T>;
        self.end = (block.addr + block.size()) as *mut Node<T>;
        self.cap = block.size() / size_of::<Node<T>>();
        (*self.base).init(0, self.cap);
        self.order = 0;
        while 1 << self.order < size_of::<T>() {
            self.order += 1;
        }
    }

    pub unsafe fn give(&self, node_content: *mut T) -> bool {
        if self.base > node_content as *mut Node<T> || self.end < node_content as *mut Node<T> {
            return false;
        }
        let node = node_content as *mut Node<T>;
        let index = (node as usize - self.base as usize) / size_of::<Node<T>>();
        (*node).reset_count(index, self.cap);
        (*node).mark_available();
        true
    }

    pub unsafe fn get(&self) -> Option<*mut T> {
        let ptr = (*self.base).get(0, self.cap);
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}

impl<T: Sized> Node<T> {
    pub unsafe fn init(&mut self, index: usize, cap: usize) -> usize {
        let mut right_count = 0;
        let mut left_count = 0;
        if let Some((left_index, right_index)) = self.children(index, cap) {
            left_count = (*(self as *mut Node<T>).offset((left_index - index) as isize))
                .init(left_index, cap);
            right_count = (*(self as *mut Node<T>).offset((right_index - index) as isize))
                .init(right_index, cap);
        }
        self.count = left_count + right_count + 1;
        self.mark_available();
        self.count()
    }

    pub unsafe fn get(&mut self, index: usize, cap: usize) -> *mut T {
        if let Some((left_index, right_index)) = self.children(index, cap) {
            let left_count =
                (*(self as *mut Node<T>).offset((left_index - index) as isize)).count();
            let right_count =
                (*(self as *mut Node<T>).offset((right_index - index) as isize)).count();
            if left_count > right_count {
                (*(self as *mut Node<T>).offset((left_index - index) as isize)).get(left_index, cap)
            } else if right_count > 0 {
                (*(self as *mut Node<T>).offset((right_index - index) as isize))
                    .get(right_index, cap)
            } else if self.available() {
                self.mark_unavailable();
                self.decrement_count(index);
                &mut self.content as *mut T
            } else {
                0 as *mut T
            }
        } else if self.available() {
            self.mark_unavailable();
            self.decrement_count(index);
            &mut self.content as *mut T
        } else {
            0 as *mut T
        }
    }

    pub unsafe fn reset_count(&mut self, index: usize, cap: usize) {
        let addr = self as *mut Node<T>;
        if let Some((left_index, right_index)) = self.children(index, cap) {
            self.count = (*addr.offset((left_index - index) as isize)).count()
                + (*addr.offset((right_index - index) as isize)).count()
                + 1;
        } else {
            self.count = 1;
        }
        self.increment_count(index);
    }

    unsafe fn increment_count(&mut self, index: usize) {
        self.count += 1;
        if index > 0 {
            let previous_index: usize = (index - 1) / 2;
            let offset: isize = previous_index as isize - index as isize;
            (*(self as *mut Node<T>).offset(offset)).increment_count(previous_index)
        }
    }

    unsafe fn decrement_count(&mut self, index: usize) {
        self.count -= 1;
        if index > 0 {
            let previous_index: usize = (index - 1) / 2;
            let offset: isize = previous_index as isize - index as isize;
            (*(self as *mut Node<T>).offset(offset)).decrement_count(previous_index);
        }
    }

    fn available(&self) -> bool {
        self.count & NODE_AVAILABLE_BIT != 0
    }

    fn count(&self) -> usize {
        self.count & !NODE_AVAILABLE_BIT
    }

    fn children(&self, index: usize, cap: usize) -> Option<(usize, usize)> {
        let right_index = index * 2 + 2;
        if right_index < cap {
            Some((right_index - 1, right_index))
        } else {
            None
        }
    }

    fn mark_available(&mut self) {
        self.count |= NODE_AVAILABLE_BIT;
    }

    fn mark_unavailable(&mut self) {
        self.count &= !NODE_AVAILABLE_BIT;
    }
}
