use acpi::madt::MADT_IOAPIC;
use mem::allocator::ALLOCATOR;


pub struct IOApic {
    id: u8,
    addr: u32,
    int_base: u32,
    max_irq: u32
}

impl IOApic {
        const IOAPICREGSEL: u8 = 0x0;
        const IOAPICWIN: u8 = 0x10;

        const IOAPICID: u8 = 0x0;
        const IOAPICVER: u8 = 0x1;
        const IOAPICARB: u8 = 0x2;

        pub unsafe fn new(io_apic_ptr: *const MADT_IOAPIC) -> IOApic {
            let mut io_apic = IOApic {
                id: (*io_apic_ptr).ioapic_id,
                addr: (*io_apic_ptr).ioapic_addr,
                int_base: (*io_apic_ptr).global_system_interrupt_base,
                max_irq: 0
            };
            if let Err(error) = ALLOCATOR.id_map(io_apic.addr as usize, mem::frame::FRAME_SIZE) {
                panic!("Unable to map IOAPIC {:?}", error);
            }
            let ver = io_apic.read_reg(Self::IOAPICVER);
            io_apic.max_irq = (ver >> 16) & 0xff;
            io_apic
        }

        unsafe fn get_selector(&self, reg: u8) -> u32 {
            let mut selector: u32;
            asm!("mov eax, [$0]"
                : "={eax}"(selector)
                : "r"(self.addr + Self::IOAPICREGSEL as u32)
                :: "intel");
            (selector & !0xff) | reg as u32
        }

        unsafe fn read_reg(&self, reg: u8) -> u32 {
            let value: u32;
            let selector = self.get_selector(reg);
            asm!("mov [ebx], ecx 
                mov eax, [edx]"
                : "={eax}"(value)
                : "{ebx}"(self.addr + Self::IOAPICREGSEL as u32), 
                "{ecx}"(selector),
                "{edx}"(self.addr + Self::IOAPICWIN as u32)
                :: "intel", "volatile");
            value
        }

        unsafe fn write_reg(&self, reg: u8, value: u32) {
            let selector = self.get_selector(reg);
            asm!("mov [eax], ebx
                mov [ecx], edx"
                :: "{eax}"(self.addr + Self::IOAPICREGSEL as u32),
                "{ebx}"(selector),
                "{ecx}"(self.addr + Self::IOAPICWIN as u32),
                "{edx}"(value)
                :: "intel");
        }

        unsafe fn write_rtbl(&self, id: u8, value: u64) {
            self.write_reg(id, (value & 0xffff) as u32);
            self.write_reg(id + 1, (value >> 32) as u32);
        }
}
