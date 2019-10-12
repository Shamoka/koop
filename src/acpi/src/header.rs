#[repr(C, packed)]
pub struct Header {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_tabel_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32
}

impl Header {
    pub unsafe fn validate(&self) -> bool {
        let ptr = self as *const Header as *const u8;
        let mut sum: usize = 0;
        for i in 0..self.length {
            sum += *(ptr.offset(i as isize) as *const u8) as usize;
        }
        sum & 0xff == 0
    }
}
