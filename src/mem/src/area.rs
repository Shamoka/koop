use crate::addr::Addr;
use crate::frame;

#[derive(Debug, Copy, Clone)]
pub struct Area {
     pub base: Addr,
     pub len: usize,
}

pub struct AreaIter {
    pos: usize,
    end: usize,
}

impl Area {
    pub const fn new(base: usize, len: usize) -> Area {
        Area {
            base: Addr::new(base),
            len: len
        }
    }

    pub fn order(&self) -> usize {
        for i in 0..64 {
            if self.len < 1 << i {
                return i
            }
        }
        63
    }

    pub fn pages(&self) -> AreaIter {
        AreaIter {
            pos: self.base.addr & !(frame::FRAME_SIZE - 1),
            end: self.base.addr + self.len,
        }
    }
}

impl Iterator for AreaIter {
    type Item = Addr;

    fn next(&mut self) -> Option<Addr> {
        if self.pos >= self.end {
            return None;
        }
        let mut addr = Addr::new(self.pos);
        addr.to_valid();
        self.pos += frame::FRAME_SIZE;
        Some(addr)
    }
}
