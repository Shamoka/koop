pub struct Info {
    addr: usize,
    size: u32,
    entry_size: u32,
    _entry_version: u32,
}

pub struct InfoIter<'a> {
    info: &'a Info,
    addr: usize,
    count: i32,
}

pub struct Entry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
}

impl Info {
    pub fn new(tag: &super::Tag) -> Info {
        if tag.tag_type != super::TagType::MemMap as u32 {
            panic!("Invalid 'memory map' tag");
        }
        Info {
            addr: tag.addr,
            size: unsafe { *((tag.addr + 4) as *const u32) },
            entry_size: unsafe { *((tag.addr + 8) as *const u32) },
            _entry_version: unsafe { *((tag.addr + 12) as *const u32) },
        }
    }

    pub fn entries(&self) -> InfoIter {
        InfoIter {
            info: self,
            addr: self.addr + 16,
            count: ((self.size - 16) / self.entry_size) as i32,
        }
    }
}

impl<'a> Iterator for InfoIter<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        match self.count > 0 {
            true => {
                self.count -= 1;
                let entry = Entry {
                    base_addr: unsafe { *(self.addr as *const u64) },
                    length: unsafe { *((self.addr + 8) as *const u64) },
                    entry_type: unsafe { *((self.addr + 16) as *const u32) },
                };
                self.addr += self.info.entry_size as usize;
                Some(entry)
            }
            false => None,
        }
    }
}
