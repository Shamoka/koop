pub struct Info {
    pub mem_lower: u32,
    pub mem_upper: u32,
}

impl Info {
    pub fn new(tag: &super::Tag) -> Info {
        if tag.tag_type != super::TagType::BasicMemInfo as u32 {
            panic!("Invalid 'basic memory information' tag");
        }
        Info {
            mem_lower: unsafe { *((tag.addr + 8) as *const u32) },
            mem_upper: unsafe { *((tag.addr + 12) as *const u32) },
        }
    }
}
