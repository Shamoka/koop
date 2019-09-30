use crate::frame;
use crate::table::{PML4, TableLevel};
use crate::AllocError;
use crate::area::Area;
use crate::allocator::PML4_ADDR;
use crate::addr::{AddrType, Addr};
use crate::entry;
use crate::entry::Entry;
use crate::UPPER_MEMORY_BOUND;

const NEW_PML4: Addr = Addr::new(0xdeadbeef000, AddrType::Virtual);

pub struct Allocator {
    frame_allocator: frame::Allocator,
    pml4: PML4
}

impl Allocator {
    pub fn new(mb2: multiboot2::Info) -> Result<Allocator, AllocError> {
        let mut allocator = Allocator {
            frame_allocator: frame::Allocator::new(mb2),
            pml4: PML4::new(&PML4_ADDR, 511),
        };
        let (new_pml4, pml4_frame) = match allocator.create_new_pml4() {
            Ok(res) => res,
            Err(error) => return Err(error)
        };
        unsafe {
            asm::x86_64::tlb::flush();
            asm::x86_64::efer::set_bit(asm::x86_64::efer::BIT_NXE);
        }
        if let Err(error) = allocator.remap_kernel(new_pml4) {
            return Err(error);
        }
        if let Err(error) = allocator.remap_low_memory(new_pml4) {
            return Err(error);
        }
        unsafe {
            asm::x86_64::tlb::update(pml4_frame.base.addr);
        }
        allocator.pml4 = new_pml4;
        Ok(allocator)
    }

    fn create_new_pml4(&mut self) -> Result<(PML4, frame::Frame), AllocError> {
        let pml4_frame = match self.frame_allocator.alloc() {
            Ok(frame) => frame,
            Err(error) => return Err(error)
        };
        if let Err(error) = self.pml4.map_frame(&NEW_PML4,
                                                Entry::new(pml4_frame.base.addr, 
                                                           entry::FLAG_WRITABLE 
                                                           | entry::FLAG_PRESENT),
                                                           &mut self.frame_allocator) {
                return Err(error);
            }
        let mut new_pml4 = PML4::new(&NEW_PML4, 510);
        new_pml4.flush(0, 511);
        new_pml4.set_entry(510, Entry::new(pml4_frame.base.addr,
                                           entry::FLAG_WRITABLE
                                           | entry::FLAG_PRESENT));
        self.pml4.set_entry(510, Entry::new(pml4_frame.base.addr,
                                           entry::FLAG_WRITABLE
                                           | entry:: FLAG_PRESENT));
        Ok((new_pml4, pml4_frame))
    }

    fn remap_kernel(&mut self, mut new_pml4: PML4) -> Result<(), AllocError> {
        for section in self.frame_allocator.mb2.get_elf_sections().unwrap() {
            let area = Area::new(section.sh_addr, section.sh_size, AddrType::Virtual);
            for addr in area.pages() {
                if let Err(error) = 
                    new_pml4.map_frame(&addr,
                                       Entry::from_elf(addr.addr, section.sh_flags),
                                       &mut self.frame_allocator) {
                        return Err(error);
                    }
            }
        }
        Ok(())
    }

    fn remap_low_memory(&mut self, mut new_pml4: PML4) -> Result<(), AllocError> {
            let area = Area::new(0, UPPER_MEMORY_BOUND, AddrType::Virtual);
            for addr in area.pages() {
                if let Err(error) = 
                    new_pml4.map_frame(&addr,
                                       Entry::new(addr.addr,
                                                  entry::FLAG_PRESENT | entry::FLAG_WRITABLE),
                                                  &mut self.frame_allocator) {
                        return Err(error);
                    }
            }
        Ok(())
    }
}
