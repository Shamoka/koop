use acpi::madt::MADT_IOAPIC;
use mem::allocator::ALLOCATOR;


pub struct IOApic {
    id: u8,
    addr: u32,
    int_base: u32,
    max_irq: u32
}

impl IOApic {
    const IOAPICREGSEL: u32 = 0x0;
    const IOAPICWIN: u32 = 0x10;
    const IOAPICID: u32 = 0x0;
    const IOAPICVER: u32 = 0x1;
    const IOAPICARB: u32 = 0x2;

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

    unsafe fn read_reg(&self, reg: u32) -> u32 {
        let value: u32;
        let mut selector: u32;
        asm!("mov eax, [$0]" : "={eax}"(selector) : "r"(self.addr + Self::IOAPICREGSEL) :: "intel");
        selector = (selector & !0xff) | reg as u32;
        asm!("mov [ebx], ecx 
            mov eax, [edx]"
            : "={eax}"(value)
            : "{ebx}"(self.addr + Self::IOAPICREGSEL), "{edx}"(self.addr + Self::IOAPICWIN), "{ecx}"(selector)
            :: "intel", "volatile");
        value
    }
}
