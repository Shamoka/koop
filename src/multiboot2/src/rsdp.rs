use crate::Tag;

pub enum Info {
    V1(RSDPv1),
    V2(RSDPv2)
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RSDPv1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    address: u32
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RSDPv2 {
    v1: RSDPv1,
    length: u32,
    address: u64,
    checksum: u8,
    _res: [u8; 3]
}

impl Info {
    pub fn new_v1(tag: &Tag) -> Info {
        unsafe {
            let r1 = *((tag.addr + 8) as *const RSDPv1);
            Info::V1(r1)
        }
    }

    pub fn new_v2(tag: &Tag) -> Info {
        unsafe {
            let r2 = *((tag.addr + 8) as *const RSDPv2);
            Info::V2(r2)
        }
    }

    pub fn addr(&self) -> usize {
        match *self {
            Info::V1(v1) => v1.address as usize,
            Info::V2(v2) => v2.address as usize
        }
    }
}
