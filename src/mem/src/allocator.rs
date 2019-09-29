use crate::addr::{Addr, AddrType};
use crate::AllocError;
use crate::area::Area;
use crate::stage1;

use spinlock::Mutex;

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};
use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

pub const TMP_STACK_AREA: Area = Area::new(0o000000_001_000_000_000_0000, 0x1000, AddrType::Virtual);
pub const MEMORY_MAP_AREA: Area = Area::new(0o000000_002_000_000_000_0000, 0x1000, AddrType::Virtual);
pub const PML4_ADDR: Addr = Addr::new(0xffff_ffff_ffff_f000, AddrType::Virtual);

pub struct Allocator {
    internal: UnsafeCell<Stage>,
    mutex: Mutex<()>
}

enum Stage {
    Stage0,
    Stage1(stage1::Allocator)
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            internal: UnsafeCell::new(Stage::Stage0),
            mutex: Mutex::new(())
        }
    }

    pub unsafe fn memmap(&self, area: &Area) -> Result<(), AllocError> {
        let _lock = self.mutex.lock();
        match &mut *self.internal.get() {
            Stage::Stage0 => Err(AllocError::Uninitialized),
            Stage::Stage1(allocator) => allocator.memmap(area),
        }
    }

    pub unsafe fn init(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        let _lock = self.mutex.lock();
        match &mut *self.internal.get() {
            Stage::Stage0 => {
                self.stage1(mb2)
            }
            Stage::Stage1(_allocator) => {
                Err(AllocError::InvalidInit)
            }
        }
    }

    unsafe fn stage1(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        match stage1::Allocator::new(mb2) {
            Ok(allocator) => {
                *self.internal.get() = Stage::Stage1(allocator);
                Ok(())
            },
            Err(error) => Err(error)
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
