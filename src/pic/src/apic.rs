use acpi::RSDT;
use acpi::madt::*;
use crate::ioapic::IOApic;

extern crate alloc;

use alloc::vec::Vec;

const SPURIOUS: u8 = 0xff;

pub struct LocalApic {
    proc_id: u8,
    id: u8,
    enabled: bool
}

pub struct Apic {
    local_apics: Vec<LocalApic>,
    io_apics: Vec<IOApic>

}

impl Apic {
    pub const fn new() -> Apic {
        Apic {
            local_apics: Vec::<LocalApic>::new(),
            io_apics: Vec::<IOApic>::new()
        }
    }

    pub unsafe fn handle_local_apic(&mut self, apic_ptr: *const MADT_LAPIC) {
        self.local_apics.push(LocalApic {
            proc_id: (*apic_ptr).proc_id,
            id: (*apic_ptr).apic_id,
            enabled: (*apic_ptr).flags == 1
        });
    }

    pub unsafe fn handle_io_apic(&mut self, io_apic_ptr: *const MADT_IOAPIC) {
        self.io_apics.push(IOApic::new(io_apic_ptr));
    }
}

pub unsafe fn init(rdsp: usize) -> bool {
    let rsdt_ptr = rdsp as *const RSDT;
    if (*rsdt_ptr).validate() == false {
        panic!("Invalid RSDT");
    }
    let mut apic = Apic::new();
    if let Some(madt_ptr) = (*rsdt_ptr).find_table("APIC") {
        if (*(madt_ptr as *const MADT)).flags == 1 {
            asm::x86_64::apic::disable_pic();
        }
        for entry in (*(madt_ptr as *const MADT)).entries() {
            match entry {
                Entry::EntryLAPIC(ptr) => apic.handle_local_apic(ptr),
                Entry::EntryIOAPIC(ptr) => apic.handle_io_apic(ptr),
                Entry::EntryISO(_) => vga::println!("Interrupt source override found"),
                Entry::EntryNMI(_) => vga::println!("Non maskable interrupt found"),
                Entry::EntryLAPICOverride(_) => vga::println!("Local APIC override found"),
                _ => vga::println!("Unknown entry in MADT")
            }
        }
        if asm::x86_64::instruction::cpuid::check_apic() {
            let apic = asm::x86_64::reg::apic_base::enable();
            apic.set_sivr(SPURIOUS);
            return true
        }
    }
    false
}
