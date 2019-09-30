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

    pub fn pages(&self) -> AreaIter {
        AreaIter {
            pos: self.base.addr,
            end: self.base.addr + self.len,
        }
    }

    pub fn contains(&self, addr: &Addr) -> bool {
        self.base.addr <= addr.addr
            && self.base.addr + self.len > addr.addr
    }

    pub fn split(&mut self, size: usize) -> Option<Area> {
        if self.len < size {
            return None;
        }
        let new_area = Area::new(self.base.addr + self.len - size, size);
        self.len -= size;
        Some(new_area)
    }

    pub fn overlap(&self, other: &Area) -> bool {
        if other.len == 0 {
            return false;
        }
        if self.base.addr <= other.base.addr
            && self.base.addr + self.len > other.base.addr {
                return true;
            }
        if self.base.addr >= other.base.addr
            && other.base.addr + other.len > self.base.addr {
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
        let mut addr = Addr::new(self.pos);
        addr.to_valid();
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
        self.base.addr.cmp(&other.base.addr)
    }
}

impl core::cmp::PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.base.addr == other.base.addr
    }
}

impl core::cmp::Eq for Area { }
