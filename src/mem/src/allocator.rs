use crate::frame;
use crate::table::PML4;
use crate::table::TableLevel;
use crate::addr::Addr;
use crate::AllocError;

use spinlock::Mutex;

use core::cell::UnsafeCell;

pub static ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());

pub struct Allocator {
    frame_allocator: UnsafeCell<Option<frame::Allocator>>,
    pml4: UnsafeCell<Option<PML4>>
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            frame_allocator: UnsafeCell::new(None),
            pml4: UnsafeCell::new(None)
        }
    }

    pub unsafe fn init(&self, pml4: usize, mb2: multiboot2::Info) {
        *(self.frame_allocator.get()) = Some(frame::Allocator::new(mb2));
        *(self.pml4.get()) = Some(PML4::new(Addr::new(pml4)));
    }

    pub fn map_page(&self, addr: Addr) -> Result<(), AllocError> {
        unsafe {
            match &mut *self.frame_allocator.get() {
                Some(frame_allocator) => {
                    match &mut *self.pml4.get() {
                        Some(pml4) => pml4.map_addr(addr, frame_allocator),
                        None => Err(AllocError::Uninitialized)
                    }
                },
                None => Err(AllocError::Uninitialized)
            }
        }
    }
}
