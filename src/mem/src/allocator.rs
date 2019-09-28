use crate::frame;
use crate::table::PML4;
use crate::table::TableLevel;
use crate::addr::Addr;
use crate::AllocError;
use crate::area::Area;
use crate::stack::Stack;
use crate::stack;

use spinlock::Mutex;

use core::cell::UnsafeCell;

pub static TMP_ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());

pub struct Allocator {
    frame_allocator: UnsafeCell<Option<frame::Allocator>>,
    pml4: UnsafeCell<Option<PML4>>,
    stack: UnsafeCell<Option<Stack>>,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            frame_allocator: UnsafeCell::new(None),
            pml4: UnsafeCell::new(None),
            stack: UnsafeCell::new(None)
        }
    }

    pub unsafe fn init(&self, pml4: usize, mb2: multiboot2::Info) {
        *(self.frame_allocator.get()) = Some(frame::Allocator::new(mb2));
        let mut pml4_table = PML4::new(&Addr::new(pml4));
        pml4_table.flush(1, 510);
        *(self.pml4.get()) = Some(pml4_table);
        if let Some(frame_allocator) =  &mut *self.frame_allocator.get() {
            if let Some(pml4) =  &mut *self.pml4.get() {
                for page in stack::DEFAULT_TEMP_STACK_AREA.pages() {
                    if let Err(error) = pml4.map_addr(&page, frame_allocator) {
                        panic!("{:?}", error);
                    }
                }
            }
        }
        *(self.stack.get()) = Some(Stack::new());
    }

    unsafe fn map_page(&self, addr: &Addr) -> Result<(), AllocError> {
        if let Some(frame_allocator) = &mut *self.frame_allocator.get() {
            if let Some(pml4) = &mut *self.pml4.get() {
                if let Err(error) = pml4.map_addr(addr, frame_allocator) {
                    return Err(error);
                }
                return Ok(());
            }
        }
        Err(AllocError::Uninitialized)
    }

    pub fn map_area(&self, area: &Area) -> Result<(), AllocError> {
        unsafe {
            match &mut *self.stack.get() {
                Some(stack) => {
                    if stack.contains_area(area) {
                        return Err(AllocError::InUse);
                    }
                },
                None => return Err(AllocError::Uninitialized)
            };
            for page in area.pages() {
                let to_map = Addr::to_valid(&page);
                if to_map.bits.value & (0o777 << 39) == (0o777 << 39) {
                    return Err(AllocError::Forbidden);
                }
                if let Err(error) = self.map_page(&to_map) {
                    return Err(error);
                }
            }
            match &mut *self.stack.get() {
                Some(stack) => {
                    if let Err(error) = stack.push(area, self) {
                        return Err(error);
                    }
                },
                None => return Err(AllocError::Uninitialized)
            };
            Ok(())
        }
    }
}
