#![no_std]

use core::mem::size_of;

mod table;

use crate::table::Header;

#[repr(C, packed)]
pub struct RSDT {
    header: Header
}

impl RSDT {
    pub unsafe fn validate(&self) -> bool {
        self.header.validate()
    }

    pub unsafe fn find_table(&self, signature: &str) -> Option<*const Header> {
        let ptr = (self as *const RSDT).offset(1) as *const u32;
        let size = (self.header.length as usize - size_of::<Header>()) / size_of::<u32>();
        for i in 0..size {
            let table_ptr = (*ptr.offset(i as isize)) as *const Header;
            if (*table_ptr).signature == signature.as_bytes() {
                return Some(table_ptr);
            }
        }
        None
    }
}
