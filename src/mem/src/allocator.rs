use crate::frame;
use crate::table::PML4;
use crate::table::TableLevel;
use crate::addr::Addr;
use crate::AllocError;
use crate::area::Area;
use crate::area;
use crate::stack::Stack;

use spinlock::Mutex;

use core::cell::UnsafeCell;

pub static ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());
pub const TEMP_STACK_AREA: Area = Area::new(0o000000_001_000_000_000_0000,
                                            0x1000,
                                            area::Alignment::Page);
pub const PML4_ADDR: Addr = Addr::new(0xffff_ffff_ffff_f000);

pub struct Allocator {
    internal: UnsafeCell<Stage>
}

pub struct Stage0Allocator {
    frame_allocator: frame::Allocator,
    stack: Stack,
    pml4: PML4
}

enum Stage {
    Uninitialized,
    Stage0(Stage0Allocator)
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
                Stage::Stage0(allocator) => allocator.memmap(area),
                Stage::Uninitialized => Err(AllocError::Uninitialized)
            }
        }
    }

    pub unsafe fn stage0(&self, mb2: multiboot2::Info) -> Result<(), AllocError> {
        match Stage0Allocator::new(mb2) {
            Ok(allocator) => {
                *self.internal.get() = Stage::Stage0(allocator);
                Ok(())
            },
            Err(error) => Err(error)
        }
    }
}

impl Stage0Allocator {
    pub fn new(mb2: multiboot2::Info) -> Result<Stage0Allocator, AllocError> {
        let mut allocator = Stage0Allocator {
            frame_allocator: frame::Allocator::new(mb2),
            pml4: PML4::new(&PML4_ADDR),
            stack: Stack::new(&TEMP_STACK_AREA)

        };
        allocator.pml4.flush(1, 510);
        for page in TEMP_STACK_AREA.pages() {
            match allocator.pml4.map_addr(&page, &mut allocator.frame_allocator) {
                Ok(()) => (),
                Err(error) => return Err(error)
            };
        }
        Ok(allocator)
    }

    pub fn memmap(&mut self, area: &Area) -> Result<(), AllocError> {
        if self.stack.contains_area(area) {
            return Err(AllocError::InUse);
        }
        if let Err(addr) = self.stack.push(area) {
            if let Err(error) = self.pml4.map_addr(&addr, &mut self.frame_allocator) {
                return Err(error);
            }
            if let Err(_error) = self.stack.push(area) {
                return Err(AllocError::Uninitialized);
            }
        }
        for page in area.pages() {
            let to_map = Addr::to_valid(&page);
            if to_map.bits.value & (0o777 << 39) == (0o777 << 39) {
                return Err(AllocError::Forbidden);
            }
            if let Err(error) = self.pml4.map_addr(&to_map, &mut self.frame_allocator) {
                return Err(error);
            }
        }
        Ok(())
    }
}
