#![no_std]
#![feature(asm)]

use idt::IDT;

mod pic;
mod apic;
mod ioapic;

pub unsafe fn init(rdsp: usize) {
    IDT.init();
    if apic::init(rdsp) == false {
        pic::init();
    }
}
