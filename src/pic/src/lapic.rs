use acpi::madt::MADT_LAPIC;
use mem::allocator::ALLOCATOR;

pub struct LocalApic {
    pub proc_id: u8,
    pub id: u8,
    pub enabled: bool,
    nmi: u64,
    nmi_pin: u8,
    base: Option<usize>,
}

impl LocalApic {
    const ISO_FLAG_TRIG: u8 = 0b11 << 2;
    const ISO_FLAG_POL: u8 = 0b11;

    const LINT_FLAG_LOW: u64 = 1 << 13;
    const LINT_FLAG_LVL: u64 = 1 << 15;

    const SIVR: usize = 0xff;

    const APIC_SIV_REG: usize = 0xf0;
    const APIC_ID_REG: usize = 0x20;
    const LVT_LINT0_REG: usize = 0x350;
    const LVT_LINT1_REG: usize = 0x360;

    pub unsafe fn new(ptr: *const MADT_LAPIC) -> LocalApic {
        LocalApic {
            proc_id: (*ptr).proc_id,
            id: (*ptr).apic_id,
            enabled: (*ptr).flags == 1,
            nmi: 0,
            nmi_pin: 0,
            base: None,
        }
    }

    pub unsafe fn map(&mut self) {
        if self.base.is_some() {
            panic!("Trying to map an already mapped local APIC");
        }
        let base = asm::x86_64::reg::apic_base::get_base();
        if let Err(error) = ALLOCATOR.id_map(base, mem::frame::FRAME_SIZE) {
            panic!("Unable to map local APIC {:?}", error);
        }
        self.base = Some(base);
    }

    pub unsafe fn set_nmi(&mut self, nmi_pin: u8, flags: u8) {
        self.nmi = (0b100 << 8) as u64;
        if flags & Self::ISO_FLAG_POL == Self::ISO_FLAG_POL {
            self.nmi |= Self::LINT_FLAG_LOW;
        }
        if flags & Self::ISO_FLAG_TRIG == Self::ISO_FLAG_TRIG {
            self.nmi |= Self::LINT_FLAG_LVL;
        }
        self.nmi_pin = nmi_pin;
    }

    pub unsafe fn setup_nmi(&self) {
        if let Some(base) = self.base {
            if self.nmi != 0 {
                let lvt_addr: usize;
                if self.nmi_pin == 0 {
                    lvt_addr = base + Self::LVT_LINT0_REG;
                } else if self.nmi_pin == 1 {
                    lvt_addr = base + Self::LVT_LINT1_REG;
                } else {
                    panic!("Unknown LINT pin for NMI");
                }
                asm!("mov [ecx], eax"
                    :: "{ecx}"(lvt_addr), "{eax}"(self.nmi)
                    :: "intel", "volatile");
            }
        } else {
            panic!("Accessing an unmapped local Apic");
        }
    }

    pub unsafe fn set_sivr(&self) {
        let mut sivr: usize;

        if let Some(base) = self.base {
            asm!("mov rax, [rcx]"
                : "={rax}"(sivr)
                : "{rcx}"(base + Self::APIC_SIV_REG)
                :: "intel", "volatile");
            sivr &= !0xFF;
            sivr |= 1 << 8;
            sivr |= (Self::SIVR & 0xFF) as usize;
            asm!("mov [rcx], rax"
                :: "{rax}"(sivr),
                "{rcx}"(base + Self::APIC_SIV_REG)
                :: "intel", "volatile");
        } else {
            panic!("Accessing an unmapped local Apic");
        }
    }
}
