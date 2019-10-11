#![no_std]

use idt::IDT;
use mem::allocator::ALLOCATOR;
use multiboot2;

mod pic;
mod apic;

pub unsafe fn init(use_apic: bool, rdsp: usize) {
    IDT.init();
    // find MADT
    // find IO APICs
    // find LAPICs
    if use_apic && asm::x86_64::instruction::cpuid::check_apic() {
        apic::init();
    } else {
        pic::init();
    }
}
