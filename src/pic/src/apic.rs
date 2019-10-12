use acpi::RSDT;
use acpi::madt::{MADT, Entry};

const SPURIOUS: u8 = 0xff;

pub unsafe fn init(rdsp: usize) -> bool {
    let rsdt_ptr = rdsp as *const RSDT;
    if (*rsdt_ptr).validate() == false {
        panic!("Invalid RSDT");
    }
    if let Some(madt_ptr) = (*rsdt_ptr).find_table("APIC") {
        for entry in (*(madt_ptr as *const MADT)).entries() {
            match entry {
                Entry::EntryLocalAPIC(_) => vga::println!("Local APIC found"),
                Entry::EntryIOAPIC(_) => vga::println!("IO APIC found"),
                _ => vga::println!("Unimplemented entry")
            }
        }
        if asm::x86_64::instruction::cpuid::check_apic() {
            asm::x86_64::apic::disable_pic();
            let apic = asm::x86_64::reg::apic_base::enable();
            apic.set_sivr(SPURIOUS);
            return true
        }
    }
    false
}
