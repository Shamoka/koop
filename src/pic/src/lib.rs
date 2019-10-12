#![no_std]

use idt::IDT;

mod pic;
mod apic;

pub unsafe fn init(rdsp: usize) {
    IDT.init();
    if apic::init(rdsp) == false {
        pic::init();
    }
}
