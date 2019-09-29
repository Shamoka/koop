use crate::addr::Addr;
use crate::frame;

#[derive(Debug, Copy, Clone)]
pub struct Area {
     pub base: Addr,
     pub len: usize,
}

pub struct AreaIter {
    pos: usize,
    end: usize
}

impl Area {
    pub const fn new(base: usize, len: usize) -> Area {
        Area {
            base: Addr::new(base),
            len: len
        }
    }

    pub fn pages(&self) -> AreaIter {
        AreaIter {
            pos: self.base.bits.value,
            end: self.base.bits.value + self.len
        }
    }

    pub fn contains(&self, addr: &Addr) -> bool {
        self.base.bits.value <= addr.bits.value
            && self.base.bits.value + self.len > addr.bits.value
    }

    pub fn overlap(&self, other: &Area) -> bool {
        if other.len == 0 {
            return false;
        }
        if self.base.bits.value <= other.base.bits.value
            && self.base.bits.value + self.len > other.base.bits.value {
                return true;
            }
        if self.base.bits.value >= other.base.bits.value
            && other.base.bits.value + other.len > self.base.bits.value {
                return true;
            }
        return false;
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

impl core::cmp::PartialOrd for Area {
    fn partial_cmp(&self, other: &Area) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::cmp::Ord for Area {
    fn cmp(&self, other: &Area) -> core::cmp::Ordering {
        self.base.bits.value.cmp(&other.base.bits.value)
    }
}

impl core::cmp::PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.base.bits.value == other.base.bits.value
    }
}

impl core::cmp::Eq for Area { }
