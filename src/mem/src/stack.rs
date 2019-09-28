use crate::frame;
use crate::area::Area;
use crate::addr::Addr;

pub struct Stack {
    top: *mut Area,
    len: usize,
    pos: usize,
}

impl Stack {
    pub fn new(area: &Area) -> Stack {
        Stack {
            top: area.base.bits.value as *mut Area,
            len: area.len,
            pos: 0,
        }
    }

    pub fn contains_area(&self, area: &Area) -> bool {
        unsafe {
            for i in 0..self.pos {
                if (*self.top.offset(i as isize)).overlap(area) {
                    return true;
                }
            }
        }
        false
    }

    pub fn push(&mut self, value: &Area) -> Result<(), Addr> {
        if self.pos >= self.len / core::mem::size_of::<Area>() {
            self.len += frame::FRAME_SIZE;
            return Err(Addr::new(self.top as usize + self.len - frame::FRAME_SIZE));
        }
        unsafe {
            *(self.top.offset(self.pos as isize)) = *value;
            self.pos += 1;
        }
        Ok(())
    }
}
