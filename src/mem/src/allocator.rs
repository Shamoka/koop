use crate::addr::{Addr, AddrType};
use crate::AllocError;
use crate::area::Area;
use crate::stage1;

use spinlock::Mutex;

use core::cell::UnsafeCell;

pub static ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());

pub const TMP_STACK_AREA: Area = Area::new(0o000000_001_000_000_000_0000, 0x1000, AddrType::Virtual);
pub const MEMORY_MAP_AREA: Area = Area::new(0o000000_002_000_000_000_0000, 0x1000, AddrType::Virtual);
pub const PML4_ADDR: Addr = Addr::new(0xffff_ffff_ffff_f000, AddrType::Virtual);

pub struct Allocator {
    internal: UnsafeCell<Stage>
}

enum Stage {
    Stage0,
    Stage1(stage1::Allocator)
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            internal: UnsafeCell::new(Stage::Stage0)
        }
    }

    pub fn memmap(&self, area: &Area) -> Result<(), AllocError> {
        unsafe {
            match &mut *self.internal.get() {
                Stage::Stage0 => Err(AllocError::Uninitialized),
                Stage::Stage1(allocator) => allocator.memmap(area),
            }
        }
    }

    pub unsafe fn stage1(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        match &*self.internal.get() {
            Stage::Stage0 => {
                match stage1::Allocator::new(mb2) {
                    Ok(allocator) => {
                        *self.internal.get() = Stage::Stage1(allocator);
                        Ok(())
                    },
                    Err(error) => Err(error)
                }
            },
            _ => Err(AllocError::InvalidInit)
        }
    }
}
