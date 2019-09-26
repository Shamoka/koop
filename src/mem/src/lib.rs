#![no_std]

pub const UPPER_MEMORY_BOUND: usize = 1 << 20;

pub mod frame;
pub mod table;
pub mod addr;
pub mod bits;

pub struct Area {
    pub base: usize,
    pub len: usize,
}

pub struct AreaIter {
    base: usize,
    len: usize,
    pos: usize
}

pub struct Allocator {
    frame_allocator: frame::Allocator,
    pml4_addr: usize
}

impl Area {
    pub fn pages(&self) -> AreaIter {
        AreaIter {
            base: self.base,
            len: self.len,
            pos: self.base
        }
    }
}

impl Iterator for AreaIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos - self.base < self.len {
            true => {
                let ret = self.pos;
                self.pos += frame::FRAME_SIZE;
                Some(ret)
            },
            false => None
        }
    }
}

impl Allocator {
    pub fn new(frame_allocator: frame::Allocator, pml4_addr: usize) -> Allocator {
        Allocator {
            frame_allocator: frame_allocator,
            pml4_addr: pml4_addr
        }
    }

    pub fn alloc(&mut self, area: &Area) {
        let mut pml4 = table::Table::new(self.pml4_addr, 4);
        for page in area.pages() {
            pml4.map(&mut self.frame_allocator, page);
        }
    }
}
