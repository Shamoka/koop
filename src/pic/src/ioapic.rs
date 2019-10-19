use acpi::madt::MADT_IOAPIC;
use mem::allocator::ALLOCATOR;

pub struct IOApic {
    pub id: u8,
    pub addr: u32,
    pub int_base: u32,
    pub max_irq: u32,
}

impl IOApic {
    const IOAPICREGSEL: u8 = 0x0;
    const IOAPICWIN: u8 = 0x10;

    const IOAPICVER: u8 = 0x1;

    const ISO_FLAG_TRIG: u8 = 0b11 << 2;
    const ISO_FLAG_POL: u8 = 0b11;

    const RTBL_FLAG_LVL: u64 = 1 << 15;
    const RTBL_FLAG_LOW: u64 = 1 << 13;

    const IRQ_BASE: u64 = 32;

    pub unsafe fn new(io_apic_ptr: *const MADT_IOAPIC, lapic_id: u8) -> IOApic {
        let mut io_apic = IOApic {
            id: (*io_apic_ptr).ioapic_id,
            addr: (*io_apic_ptr).ioapic_addr,
            int_base: (*io_apic_ptr).global_system_interrupt_base,
            max_irq: 0,
        };
        if let Err(error) = ALLOCATOR.id_map(io_apic.addr as usize, mem::frame::FRAME_SIZE) {
            panic!("Unable to map IOAPIC {:?}", error);
        }
        let ver = io_apic.read_reg(Self::IOAPICVER);
        io_apic.max_irq = (ver >> 16) & 0xff;
        io_apic.remap(lapic_id);
        io_apic
    }

    pub unsafe fn irq_override(&self, lapic_id: u8, irq_source: u8, interrupt: u8, flags: u8) {
        let mut rtbl = Self::IRQ_BASE + interrupt as u64;
        rtbl |= (lapic_id as u64 & 0xf) << 56;
        if flags & Self::ISO_FLAG_POL == Self::ISO_FLAG_POL {
            rtbl |= Self::RTBL_FLAG_LOW;
        }
        if flags & Self::ISO_FLAG_TRIG == Self::ISO_FLAG_TRIG {
            rtbl |= Self::RTBL_FLAG_LVL;
        }
        self.write_rtbl(irq_source, rtbl);
    }

    unsafe fn remap(&self, lapic_id: u8) {
        for irq in self.int_base..self.max_irq {
            let mut rtbl = Self::IRQ_BASE + irq as u64;
            rtbl |= (lapic_id as u64 & 0xf) << 56;
            self.write_rtbl(irq as u8, rtbl);
        }
    }

    unsafe fn get_selector(&self, reg: u8) -> u32 {
        let mut selector: u32;
        asm!("mov eax, [ecx]"
                : "={eax}"(selector)
                : "{ecx}"(self.addr + Self::IOAPICREGSEL as u32)
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
        self.write_reg(id, value as u32);
        self.write_reg(id + 1, (value >> 32) as u32);
    }
}
