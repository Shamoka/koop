use crate::frame;
use crate::table::PML4;
use crate::table::TableLevel;
use crate::addr::Addr;
use crate::AllocError;
use crate::area::Area;
use crate::stack::Stack;
use crate::allocator::PML4_ADDR;
use crate::allocator::TMP_STACK_AREA;

pub struct Allocator {
    frame_allocator: frame::Allocator,
    stack: Stack,
    pml4: PML4
}

impl Allocator {
    pub fn new(mb2: multiboot2::Info) -> Result<Allocator, AllocError> {
        let mut allocator = Allocator {
            frame_allocator: frame::Allocator::new(mb2),
            pml4: PML4::new(&PML4_ADDR),
            stack: Stack::new(&TMP_STACK_AREA)

        };
        allocator.pml4.flush(1, 510);
        for page in TMP_STACK_AREA.pages() {
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
