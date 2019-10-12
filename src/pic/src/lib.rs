#![no_std]

use idt::IDT;
use acpi::RSDT;

mod pic;
mod apic;

pub unsafe fn init(use_apic: bool, rdsp: usize) {
    IDT.init();
    let rsdt_ptr = rdsp as *const RSDT;
    if (*rsdt_ptr).validate() == false {
        panic!("Invalid RSDT");
    }
    if let Some(_) = (*rsdt_ptr).find_table("APIC") {
        vga::println!("Found!");
    }
    // find IO APICs
    // find LAPICs
    if use_apic && asm::x86_64::instruction::cpuid::check_apic() {
        apic::init();
    } else {
        pic::init();
    }
}
