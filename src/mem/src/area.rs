use crate::addr::Addr;
use crate::frame;

#[derive(Copy, Clone)]
pub enum Alignment {
    Page = 0x1000,
}

pub struct Area {
     pub base: Addr,
     pub len: usize,
}

pub struct AreaIter {
    pos: usize,
    end: usize
}

impl Area {
    pub const fn new(base: usize, len: usize, align: Alignment) -> Area {
        Area {
            base: Addr::new(base - base % align as usize),
            len: (len / frame::FRAME_SIZE + 1) * (frame::FRAME_SIZE),
        }
    }

    pub fn pages(&self) -> AreaIter {
        AreaIter {
            pos: self.base.bits.value,
            end: self.base.bits.value + self.len
        }
    }
}

impl Iterator for AreaIter {
    type Item = Addr;

    fn next(&mut self) -> Option<Addr> {
        if self.pos >= self.end {
            return None;
        }
        let addr = Addr::new(self.pos);
        self.pos += frame::FRAME_SIZE;
        Some(addr)
    }
}
