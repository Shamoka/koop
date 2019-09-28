use crate::allocator::Allocator;
use crate::AllocError;
use crate::frame;
use crate::area;
use crate::area::Area;

pub struct Stack {
    top: *mut Area,
    len: usize,
    pos: usize,
}

pub const DEFAULT_TEMP_STACK_AREA: Area = Area::new(0o000000_001_000_000_000_0000,
                                                    0x1000,
                                                    area::Alignment::Page);

impl Stack {
    pub fn new() -> Stack {
        Stack {
            top: DEFAULT_TEMP_STACK_AREA.base.bits.value as *mut Area,
            len: DEFAULT_TEMP_STACK_AREA.len,
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

    pub fn push(&mut self, value: &Area, allocator: &Allocator) -> Result<(), AllocError> {
        if self.pos >= self.len / core::mem::size_of::<Area>() {
            if let Err(error) = self.grow(allocator) {
                return Err(error);
            }
        }
        unsafe {
            *(self.top.offset(self.pos as isize)) = *value;
            self.pos += 1;
        }
        Ok(())
    }

    fn grow(&mut self, allocator: &Allocator) -> Result<(), AllocError> {
        let new_area = Area::new(self.top as usize + self.len,
                                 frame::FRAME_SIZE,
                                 area::Alignment::Page);
        match allocator.map_area(&new_area) {
            Ok(()) => {
                self.len += frame::FRAME_SIZE;
                Ok(())
            },
            Err(error) => Err(error)
        }
    }
}
