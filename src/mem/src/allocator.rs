use crate::addr::Addr;
use crate::AllocError;
use crate::area::Area;
use crate::area;
use crate::stage1;

use spinlock::Mutex;

use core::cell::UnsafeCell;

pub static ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());
pub const TMP_STACK_AREA: Area = Area::new(0o000000_001_000_000_000_0000,
                                            0x1000,
                                            area::Alignment::Page);
pub const PML4_ADDR: Addr = Addr::new(0xffff_ffff_ffff_f000);

pub struct Allocator {
    internal: UnsafeCell<Stage>
}

enum Stage {
    Uninitialized,
    Stage1(stage1::Allocator)
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            internal: UnsafeCell::new(Stage::Uninitialized)
        }
    }

    pub fn memmap(&self, area: &Area) -> Result<(), AllocError> {
        unsafe {
            match &mut *self.internal.get() {
                Stage::Stage1(allocator) => allocator.memmap(area),
                Stage::Uninitialized => Err(AllocError::Uninitialized)
            }
        }
    }

    pub unsafe fn stage0(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        match stage1::Allocator::new(mb2) {
            Ok(allocator) => {
                *self.internal.get() = Stage::Stage1(allocator);
                Ok(())
            },
            Err(error) => Err(error)
        }
    }
}
