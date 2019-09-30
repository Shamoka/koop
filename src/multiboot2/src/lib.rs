#![no_std]

mod basic_mem_info;
mod mem_map;
pub mod elf;

pub struct Info {
    pub base: usize,
    pub total_size: u32
}

pub struct Tag {
    addr: usize,
    tag_type: u32,
    size: u32
}

struct TagIter {
    addr: usize
}

#[repr(u32)]
#[derive(Copy, Clone)]
enum TagType {
    BasicMemInfo = 4,
    MemMap = 6,
    Elf = 9
}

impl Info {
    pub fn new(info_addr: usize) -> Info {
        Info {
            base: info_addr,
            total_size: unsafe { *(info_addr as *const u32) }
        }
    }

    fn tags(&self) -> TagIter {
        TagIter {
            addr: self.base + 8
        }
    }

    pub fn get_basic_mem_info(&self) -> Option<basic_mem_info::Info> {
        match self.tags().find(TagType::BasicMemInfo) {
            Some(tag) => Some(basic_mem_info::Info::new(&tag)),
            None => None
        }
    }

    pub fn get_mem_map(&self) -> Option<mem_map::Info> {
        match self.tags().find(TagType::MemMap) {
            Some(tag) => Some(mem_map::Info::new(&tag)),
            None => None
        }
    }
    
    pub fn get_elf_sections(&self) -> Option<elf::SectionIter> {
        match self.tags().find(TagType::Elf) {
            Some(tag) => Some(elf::Header::new(&tag).sections()),
            None => None
        }
    }
}

impl Tag {
    pub fn new(tag_addr: usize) -> Tag {
        Tag {
            addr: tag_addr,
            tag_type: unsafe { *(tag_addr as *const u32) },
            size: unsafe { *((tag_addr + 4) as *const u32) }
        }
    }
}

impl TagIter {
    pub fn find(&mut self, tag_type: TagType) -> Option<Tag> {
        for tag in self {
            if tag.tag_type == tag_type as u32 {
                return Some(tag)
            }
        }
        None
    }
}

impl Iterator for TagIter {
    type Item = Tag;

    fn next(&mut self) -> Option<Tag> {
        unsafe {
            match *(self.addr as *const u32) {
                0 => None,
                _ => {
                    let tag = Tag::new(self.addr);
                    self.addr += tag.size as usize;
                    if self.addr % 8 != 0 {
                        self.addr += 8 - self.addr % 8;
                    }
                    Some(tag)
                }
            }
        }
    }
}
