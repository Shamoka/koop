use crate::frame::Frame;
use crate::frame;
use crate::Area;

pub struct Table<'a> {
    entries: &'a mut [Entry; 512],
    level: usize
}

struct Entry {
    pub value: usize
}

impl<'a> Table<'a> {
    pub fn new(addr: usize, level: usize) -> Table<'a> {
        Table {
            entries: unsafe { &mut *(addr as *mut [Entry; 512]) },
            level: level
        }
    }

    pub fn map(&mut self, frame_allocator: &mut frame::Allocator, page: usize) {
    }
}

impl Entry {
    pub fn is_present(&self) -> bool {
        (self.value & 0b1) != 0
    }

    pub fn is_writable(&self) -> bool {
        (self.value & 0b10) != 0
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.value |= (addr & 0xffff_ffff_ffff) << 12;
    }

    pub fn set_writable(&mut self) {
        self.value |= 0b10;
    }

    pub fn set_present(&mut self) {
        self.value |= 0b1;
    }
}
