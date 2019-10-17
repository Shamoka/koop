#![no_std]
#![feature(asm)]

use idt::IDT;

mod apic;
mod ioapic;
mod lapic;
mod pic;
mod timer;

pub unsafe fn init(rdsp: usize) {
    IDT.init();
    if apic::init(rdsp) == false {
        pic::init();
    }
    let timer = timer::Timer::new();
}
