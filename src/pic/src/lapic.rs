use acpi::madt::MADT_LAPIC;
use mem::allocator::ALLOCATOR;

enum TimerMode {
    OneShot,
    _Periodic,
    TSCDeadline,
    Uninitialized,
}

pub struct LocalApic {
    pub proc_id: u8,
    pub id: u8,
    pub enabled: bool,
    nmi: u32,
    nmi_pin: u8,
    base: Option<usize>,
    timer: TimerMode,
}

impl LocalApic {
    const ISO_FLAG_TRIG: u8 = 0b11 << 2;
    const ISO_FLAG_POL: u8 = 0b11;

    const LINT_FLAG_LOW: u32 = 1 << 13;
    const LINT_FLAG_LVL: u32 = 1 << 15;

    const TIMER_MODE_ONE_SHOT: u32 = 0;
    const _TIMER_MODE_PREDIODIC: u32 = 1 << 17;
    const TIMER_MODE_TSC_DEADLINE: u32 = 0b10 << 17;

    const SIVR: u32 = 0xff;

    const APIC_SIV_REG: usize = 0xf0;
    const _APIC_ID_REG: usize = 0x20;
    const LVT_LINT0_REG: usize = 0x350;
    const LVT_LINT1_REG: usize = 0x360;
    const TIMER_REG: usize = 0x320;
    const TIMER_INTIAL_COUNT_REG: usize = 0x380;

    pub unsafe fn new(ptr: *const MADT_LAPIC) -> LocalApic {
        LocalApic {
            proc_id: (*ptr).proc_id,
            id: (*ptr).apic_id,
            enabled: (*ptr).flags == 1,
            nmi: 0,
            nmi_pin: 0,
            base: None,
            timer: TimerMode::Uninitialized,
        }
    }

    unsafe fn write_reg(&self, reg: usize, value: u32) {
        if let Some(base) = self.base {
            asm!("mov [ecx], eax"
                :: "{eax}"(value),
                "{ecx}"(base + reg)
                :: "intel");
        } else {
            panic!("Accessing an unmapped local Apic");
        }
    }

    unsafe fn read_reg(&self, reg: usize) -> u32 {
        if let Some(base) = self.base {
            let ret: u32;
            asm!("mov eax, [ecx]"
                : "={eax}"(ret)
                : "{ecx}"(base + reg)
                :: "intel", "volatile");
            ret
        } else {
            panic!("Accessing an unmapped local Apic");
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
        self.nmi = (0b100 << 8) as u32;
        if flags & Self::ISO_FLAG_POL == Self::ISO_FLAG_POL {
            self.nmi |= Self::LINT_FLAG_LOW;
        }
        if flags & Self::ISO_FLAG_TRIG == Self::ISO_FLAG_TRIG {
            self.nmi |= Self::LINT_FLAG_LVL;
        }
        self.nmi_pin = nmi_pin;
    }

    pub unsafe fn setup_nmi(&self) {
        if self.nmi != 0 {
            if self.nmi_pin == 0 {
                self.write_reg(Self::LVT_LINT0_REG, self.nmi);
            } else if self.nmi_pin == 1 {
                self.write_reg(Self::LVT_LINT1_REG, self.nmi);
            } else {
                panic!("Unknown LINT pin for NMI");
            }
        }
    }

    pub unsafe fn set_sivr(&self) {
        let mut sivr = self.read_reg(Self::APIC_SIV_REG);
        sivr &= !0xFF;
        sivr |= 1 << 8;
        sivr |= Self::SIVR & 0xFF;
        self.write_reg(Self::APIC_SIV_REG, sivr);
    }

    pub unsafe fn init_timer(&mut self) {
        if asm::x86_64::instruction::cpuid::check_tsc_deadline() {
            self.timer = TimerMode::TSCDeadline;
            self.write_reg(Self::TIMER_REG, Self::TIMER_MODE_TSC_DEADLINE + 34);
            let time = asm::x86_64::instruction::rdtsc();
            asm::x86_64::reg::tsc_deadline::set(time + 1_000_000);
            return;
        } else {
            unimplemented!();
        }
    }
}
