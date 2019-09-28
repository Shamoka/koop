use crate::allocator::ALLOCATOR;
use crate::AllocError;
use crate::frame;
use crate::area;
use crate::area::Area;
use crate::page_manager;

pub struct Stack {
    top: *mut Area,
    len: usize,
    pos: usize,
}

const DEFAULT_TEMP_STACK_AREA: Area = Area::new(0o000000_001_000_000_000_0000,
                                                0x1000,
                                                area::Alignment::Page);

impl Stack {
    pub fn new(area: &Area) -> Result<Stack, AllocError> {
        match ALLOCATOR.lock().map_area(area) {
            Ok(()) => {
                Ok(Stack {
                    top: area.base.bits.value as *mut Area,
                    len: area.len,
                    pos: 0,
                })
            },
            Err(error) => Err(error)
        }
    }

    pub fn _contains(&self, area: &Area) -> bool {
        unsafe {
            for i in 0..self.pos {
                if (*self.top.offset(i as isize)).overlap(area) {
                    return true;
                }
            }
        }
        false
    }

    pub fn push(&mut self, value: &Area) -> Result<(), AllocError> {
        if self.pos >= self.len / core::mem::size_of::<Area>() {
            if let Err(error) = self.grow() {
                return Err(error);
            }
        }
        unsafe {
            *(self.top.offset(self.pos as isize)) = *value;
            self.pos += 1;
        }
        Ok(())
    }

    fn grow(&mut self) -> Result<(), AllocError> {
        let new_area = Area::new(self.top as usize + self.len,
                                 frame::FRAME_SIZE,
                                 area::Alignment::Page);
        match ALLOCATOR.lock().map_area(&new_area) {
            Ok(()) => {
                self.len += frame::FRAME_SIZE;
                Ok(())
            },
            Err(error) => Err(error)
        }
    }
}

impl page_manager::Manager for Stack {
    type Backend = Stack;

    fn create() -> Result<Self::Backend, page_manager::Error> {
        match Stack::new(&DEFAULT_TEMP_STACK_AREA) {
            Ok(stack) => Ok(stack),
            Err(error) => Err(page_manager::Error::Allocation(error))
        }
    }

    fn insert(&mut self, area: &Area) -> Result<(), page_manager::Error> {
        if self._contains(area) {
            return Err(page_manager::Error::InvalidPage);
        }
        match self.push(area) {
            Ok(()) => Ok(()),
            Err(error) => Err(page_manager::Error::Allocation(error))
        }
    }

    fn remove(&mut self, _area: &Area) -> Result<(), page_manager::Error> {
        Err(page_manager::Error::InvalidCall)
    }

    fn contains(&self, area: &Area) -> bool {
        self._contains(area)
    }
}
