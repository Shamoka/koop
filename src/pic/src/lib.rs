#![no_std]
#![feature(asm)]

use idt::IDT;

mod apic;
mod ioapic;
mod lapic;
mod pic;

pub unsafe fn init(rdsp: usize) {
    IDT.init();
    if let Some(_apic) = apic::init(rdsp) {
    } else {
        pic::init();
    }
}
