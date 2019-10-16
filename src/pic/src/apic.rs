use crate::ioapic::IOApic;
use acpi::madt::*;
use acpi::RSDT;

extern crate alloc;

use alloc::vec::Vec;

const SPURIOUS: u8 = 0xff;

pub struct LocalApic {
    _proc_id: u8,
    id: u8,
    enabled: bool,
}

pub struct Apic {
    local_apics: Vec<LocalApic>,
    io_apics: Vec<IOApic>,
}

impl Apic {
    pub const fn new() -> Apic {
        Apic {
            local_apics: Vec::<LocalApic>::new(),
            io_apics: Vec::<IOApic>::new(),
        }
    }

    pub unsafe fn handle_local_apic(&mut self, apic_ptr: *const MADT_LAPIC) {
        self.local_apics.push(LocalApic {
            _proc_id: (*apic_ptr).proc_id,
            id: (*apic_ptr).apic_id,
            enabled: (*apic_ptr).flags == 1,
        });
    }

    pub unsafe fn handle_io_apic(&mut self, io_apic_ptr: *const MADT_IOAPIC) {
        for local_apic in &self.local_apics {
            if local_apic.enabled {
                self.io_apics.push(IOApic::new(io_apic_ptr, local_apic.id));
                return;
            }
        }
    }

    pub unsafe fn handle_interrupt_source_override(&mut self, iso: *const MADT_ISO) {
        for local_apic in &self.local_apics {
            if local_apic.enabled {
                for io_apic in &self.io_apics {
                    if (*iso).irq_source >= io_apic.int_base as u8
                        && (*iso).irq_source <= io_apic.max_irq as u8
                    {
                        io_apic.irq_override(
                            local_apic.id,
                            (*iso).irq_source,
                            (*iso).global_system_interrupt as u8,
                            (*iso).flags as u8,
                        );
                        return;
                    }
                }
            }
        }
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
                Entry::EntryISO(ptr) => apic.handle_interrupt_source_override(ptr),
                Entry::EntryNMI(_) => vga::println!("Non maskable interrupt found"),
                Entry::EntryLAPICOverride(_) => vga::println!("Local APIC override found"),
                _ => vga::println!("Unknown entry in MADT"),
            }
        }
        if asm::x86_64::instruction::cpuid::check_apic() {
            let apic = asm::x86_64::reg::apic_base::enable();
            apic.set_sivr(SPURIOUS);
            return true;
        }
    }
    false
}
