use crate::AllocError;
use crate::stage1;
use crate::stage2;
use crate::addr::Addr;

use spinlock::Mutex;

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};
use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

pub const PML4_ADDR: Addr = Addr::new(0xffff_ffff_ffff_f000);

pub struct Allocator {
    internal: UnsafeCell<Stage>,
    mutex: Mutex<()>
}

pub enum Stage {
    Stage0,
    Stage1(stage1::Allocator),
    Stage2(stage2::Allocator)
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            internal: UnsafeCell::new(Stage::Stage0),
            mutex: Mutex::new(())
        }
    }

    pub unsafe fn init(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        let _lock = self.mutex.lock();
        match &mut *self.internal.get() {
            Stage::Stage0 => self._init(mb2),
            _ => Err(AllocError::InvalidInit)
        }
    }

    unsafe fn _init(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        match stage1::Allocator::new(mb2) {
            Ok(s1_allocator) => {
                match stage2::Allocator::new(s1_allocator) {
                    Ok(s2_allocator) => {
                        *self.internal.get() = Stage::Stage2(s2_allocator);
                        Ok(())
                    },
                    Err(error) => Err(error)
                }
            },
            Err(error) => Err(error)
        }
    }

    pub unsafe fn memalloc(&self, len: usize) -> *mut u8 {
        let _lock = self.mutex.lock();
        match &mut *self.internal.get() {
            Stage::Stage2(allocator) => allocator.alloc(len),
            _ => 0 as *mut u8
        }
    }
}

unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        0 as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    }
}
