#![no_std]

pub struct RSDT {
    addr: usize
}

impl RSDT {
    pub fn new(addr: usize) -> RSDT {
        RSDT {
            addr: addr
        }
    }
}
