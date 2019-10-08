const BITS_STACK: u16 = 0b111;
const BITS_RESERVED: u16 = 0b1111_1000;
const BIT_INTERRUPT: u16 = 0b1_0000_0000;
const BITS_ONE: u16 = 0b1110_0000_0000;
const BITS_ZERO: u16 = 0b1_0000_0000_0000;
const BITS_PRIVILEGE: u16 = 0b110_0000_0000_0000;
const BIT_PRESENT: u16 = 0b1000_0000_0000_0000;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Entry {
    ptr_low: u16,
    selector: u16,
    options: u16,
    ptr_middle: u16,
    ptr_high: u32,
    _res: u32
}

impl Entry {
    pub const fn new_empty() -> Entry {
        Entry {
            ptr_low: 0,
            selector: 0,
            options: BITS_ONE,
            ptr_middle: 0,
            ptr_high: 0,
            _res: 0
        }
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.ptr_low = addr as u16;
        self.ptr_middle = (addr >> 16) as u16;
        self.ptr_high = (addr  >> 32) as u32;
    }

    pub fn set_cs(&mut self) {
        let cs: u16;
        unsafe {
            asm!("mov %cs, $0" : "=r"(cs));
        }
        self.selector = cs;
    }

    pub fn set_present(&mut self, present: bool) {
        if present {
            self.options |= BIT_PRESENT;
        } else {
            self.options &= !BIT_PRESENT;
        }
    }
}
