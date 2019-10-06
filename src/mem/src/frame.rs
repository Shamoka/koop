use crate::addr::Addr;
use crate::AllocError;

use multiboot2;

pub const FRAME_SIZE: usize = 4096;

#[derive(Copy, Clone)]
pub struct Frame {
    pub base: Addr
}

impl Frame {
    pub fn new(value: usize) -> Frame {
        let mut frame = Frame {
            base: Addr::new(value),
        };
        frame.align();
        frame
    }

    pub fn align(&mut self) {
        self.base.addr -= self.base.addr & (FRAME_SIZE - 1);
    }
}

pub struct Allocator {
    kernel_start: usize,
    kernel_end: usize,
    memory_size: usize,
    free_base: Addr,
    pub mb2: multiboot2::Info
}

impl Allocator {
    pub fn new(mb2: multiboot2::Info) -> Allocator {
        let kstart = mb2.get_elf_sections()
            .expect("No ELF section found in multiboot2 info")
            .map(|x| x.sh_addr)
            .min().unwrap();
        let kend = mb2.get_elf_sections()
            .expect("No ELF section found in multiboot2 info")
            .map(|x| x.sh_addr + x.sh_size)
            .max().unwrap();
        let mem_size = mb2.get_basic_mem_info()
            .expect("No basic memory information in multiboot2 info")
            .mem_upper * 1024;
        Allocator {
            kernel_start: kstart as usize,
            kernel_end: kend as usize,
            memory_size: mem_size as usize,
            free_base: Addr::new(super::UPPER_MEMORY_BOUND),
            mb2: mb2
        }
    }

    pub fn alloc(&mut self) -> Result<Frame, AllocError> {
        loop {
            let frame = Frame::new(self.free_base.addr);
            self.free_base.addr += FRAME_SIZE;
            if frame.base.addr >= self.kernel_start && frame.base.addr <= self.kernel_end {
                continue;
            } else if frame.base.addr < super::UPPER_MEMORY_BOUND {
                continue;
            } else if frame.base.addr > self.memory_size {
                return Err(AllocError::OutOfMemory);
            } else {
                return Ok(frame);
            }
        }
    }
}
