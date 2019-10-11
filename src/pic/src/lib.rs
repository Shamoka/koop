#![no_std]

mod pic;
mod apic;

pub unsafe fn init(use_apic: bool) {
    if use_apic && asm::x86_64::instruction::cpuid::check_apic() {
        apic::init();
    } else {
        pic::init();
    }
}
