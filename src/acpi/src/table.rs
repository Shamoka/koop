
use core::mem::size_of;

pub struct Table {
    pub header: Header,
    pub ptr: usize
}

#[derive(Copy, Clone)]
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

impl Table {
    pub unsafe fn new(ptr: usize) -> Option<Table> {
        let table = Table {
            header: *(ptr as *const Header),
            ptr: ptr
        };
        if table.validate() {
            Some(table)
        } else {
            None
        }
    }

    unsafe fn validate(&self) -> bool {
        let mut ptr = self.ptr;
        let mut sum = 0;
        let end = self.header.length as usize / size_of::<usize>();
        let r = self.header.length as usize % size_of::<usize>();
        for _ in 0..end {
            let mut word = *(ptr as *const usize);
            for _ in 0..size_of::<usize>() {
                sum += word & 0xff;
                word >>= 8;
            }
            ptr += size_of::<usize>();
        }
        for _ in 0..r {
            let word = *(ptr as *const u8);
            sum += word as usize;
            ptr += 1;
        }
        sum & 0xff == 0
    }
}
