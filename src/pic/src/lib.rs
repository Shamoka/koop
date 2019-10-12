#![no_std]

use idt::IDT;
use acpi::RSDT;

mod pic;
mod apic;

pub unsafe fn init(use_apic: bool, rdsp: usize) {
    IDT.init();
    if let Some(rdst) = RSDT::new(rdsp) {
        rdst.find_table();
    }
    // find MADT
    // find IO APICs
    // find LAPICs
    if use_apic && asm::x86_64::instruction::cpuid::check_apic() {
        apic::init();
    } else {
        pic::init();
    }
}
