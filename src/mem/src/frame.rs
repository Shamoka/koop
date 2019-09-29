use multiboot2;

pub const FRAME_SIZE: usize = 4096;

#[derive(Copy, Clone)]
pub struct Frame {
    pub base: usize
}

impl Frame {
    pub fn new(value: usize) -> Frame {
        let mut frame = Frame { base: value };
        frame.align();
        frame
    }

    pub fn align(&mut self) {
        self.base -= self.base & (FRAME_SIZE - 1);
    }
}

pub struct Allocator {
    kernel_start: usize,
    kernel_end: usize,
    memory_size: usize,
    free_base: usize,
}

impl Allocator {
    pub fn new(mb2: &multiboot2::Info) -> Allocator {
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
            free_base: super::UPPER_MEMORY_BOUND as usize
        }
    }

    pub fn alloc(&mut self) -> Option<Frame> {
        loop {
            let frame = Frame::new(self.free_base);
            self.free_base += FRAME_SIZE;
            if frame.base >= self.kernel_start && frame.base <= self.kernel_end {
                continue;
            } else if frame.base < super::UPPER_MEMORY_BOUND {
                continue;
            } else if frame.base > self.memory_size {
                return None;
            } else {
                return Some(frame);
            }
        }
    }
}
