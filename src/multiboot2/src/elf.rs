pub const SHF_WRITE: usize = 0x1;
pub const SHF_ALLOC: usize = 0x2;
pub const SHF_EXECINSTR: usize = 0x4;

pub struct Header {
    addr: usize,
    num: u32,
    entsize: u32,
    _shndx: u32,
}

pub struct SectionIter {
    entsize: u32,
    count: i16,
    addr: usize,
}

pub struct Section {
    pub sh_type: usize,
    pub sh_flags: usize,
    pub sh_addr: usize,
    pub sh_size: usize,
}

impl Header {
    pub fn new(tag: &super::Tag) -> Header {
        unsafe {
            Header {
                addr: tag.addr,
                num: *((tag.addr + 8) as *const u32),
                entsize: *((tag.addr + 12) as *const u32),
                _shndx: *((tag.addr + 16) as *const u32),
            }
        }
    }

    pub fn sections(&self) -> SectionIter {
        SectionIter {
            count: self.num as i16,
            addr: self.addr + 20,
            entsize: self.entsize,
        }
    }
}

impl Iterator for SectionIter {
    type Item = Section;

    fn next(&mut self) -> Option<Self::Item> {
        match self.count > 0 {
            true => unsafe {
                let section = Section {
                    sh_type: *((self.addr + 0x04) as *const usize),
                    sh_flags: *((self.addr + 0x08) as *const usize),
                    sh_addr: *((self.addr + 0x10) as *const usize),
                    sh_size: *((self.addr + 0x20) as *const usize),
                };
                self.count -= 1;
                self.addr += self.entsize as usize;
                match section.sh_flags {
                    0x00 => self.next(),
                    _ => Some(section),
                }
            },
            false => None,
        }
    }
}
