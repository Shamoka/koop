pub struct Header {
    addr: usize,
    num: u32,
    entsize: u32,
    _shndx: u32
}

pub struct SectionIter {
    entsize: u32,
    count: i16,
    addr: usize
}

pub struct Section {
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_size: u64
}

impl Header {
    pub fn new(tag: &super::Tag) -> Header {
        unsafe {
            Header {
                addr: tag.addr,
                num: *((tag.addr + 8) as *const u32),
                entsize: *((tag.addr + 12) as *const u32),
                _shndx: *((tag.addr + 16) as *const u32)
            }
        }
    }

    pub fn sections(&self) -> SectionIter {
        SectionIter {
            count: self.num as i16,
            addr: self.addr + 20,
            entsize: self.entsize
        }
    }
}

impl Iterator for SectionIter {
    type Item = Section;

    fn next(&mut self) -> Option<Self::Item> {
        match self.count > 0 {
             true => {
                unsafe {
                    let section = Section {
                        sh_flags: *((self.addr + 0x08) as *const u64),
                        sh_addr: *((self.addr + 0x10) as *const u64),
                        sh_size: *((self.addr + 0x20) as *const u64)
                    };
                    self.count -= 1;
                    self.addr += self.entsize as usize;
                    Some(section)
                }
            },
            false => None
        }
    }
}
