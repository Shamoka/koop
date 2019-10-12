#![no_std]

use core::mem::size_of;

mod table;

use crate::table::{Table, Header};

pub struct RSDT {
    rsdt: Table,
}

impl RSDT {
    pub unsafe fn new(ptr: usize) -> Option<RSDT> {
        if let Some(table) = Table::new(ptr) {
            Some(RSDT {
                rsdt: table
            })
        } else {
            None
        }
    }

    pub unsafe fn find_table(&self) -> Option<Table> {
        let size = (self.rsdt.header.length as usize - size_of::<Header>()) / size_of::<u32>();
        let mut ptr = self.rsdt.ptr + size_of::<Header>();
        for _ in 0..size {
            vga::println!("{:x}", *(ptr as *const u32));
            ptr += size_of::<u32>();
        }
        None
    }
}
