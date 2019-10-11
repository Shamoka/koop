#![no_std]

use idt::IDT;
use mem::allocator::ALLOCATOR;
use multiboot2;

mod pic;
mod apic;

pub unsafe fn init(use_apic: bool, mb2: &multiboot2::Info) {
    IDT.init();
    let addr = mb2.get_rsdp().expect("No RSDP found in multiboot2 info").addr();
    if let Err(error) = ALLOCATOR.id_map(addr, mem::frame::FRAME_SIZE) {
        panic!("Can't map RSDT {:?}", error);
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
