#![no_std]
#![feature(asm)]

pub struct Port {
    addr: u16
}

impl Port {
    pub fn new(new_addr: u16) -> Port{
        Port { addr: new_addr }
    }

    pub unsafe fn write(&self, value: u8) {
        asm!("outb %al, %dx" :: "{dx}"(self.addr), "{al}"(value) :: "volatile");
    }

    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        asm!("inb %dx, %al" : "={al}"(value) : "{dx}"(self.addr) :: "volatile");
        value
    }
}
