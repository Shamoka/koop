pub mod instruction {
    pub unsafe fn int_80() {
        asm!("int $0" :: "N"(0x80) :::);
    }

    pub unsafe fn hlt() {
        asm!("hlt");
    }

    pub mod cpuid {
        const APIC_BIT: usize = 1 << 9;

        pub unsafe fn check_apic() -> bool {
            let edx: usize;
            asm!("
            mov $$0x80000001, %eax
            cpuid"
            : "={edx}"(edx) ::: "volatile");
            edx & APIC_BIT != 0
        }
    }
}

pub mod reg {
    pub mod efer {
        const ID: usize = 0xC0000080;

        pub const BIT_NXE: usize = 11;

        pub unsafe fn set_bit(bit: usize) {
            let eax: usize;
            let edx: usize;
            asm!("rdmsr" : "={edx}"(edx), "={eax}"(eax) : "{ecx}"(ID) :::"volatile");
            asm!("wrmsr" :: "{edx}"(edx), "{eax}"(eax | 1 << bit), "{ecx}"(ID) ::: "volatile");
        }
    }

    pub mod apic_base {
        const ID: usize = 0x1B;

        const APIC_ENABLE_BIT: usize = 1 << 11;

        pub unsafe fn enable() -> crate::x86_64::apic::Apic {
            let eax: usize;
            let edx: usize;
            asm!("rdmsr" : "={edx}"(edx), "={eax}"(eax) : "{ecx}"(ID) :::"volatile");
            let base = eax >> 12 + ((edx & 0b111) << 32);
            asm!("wrmsr" :: "{edx}"(edx), "{eax}"(eax | APIC_ENABLE_BIT), "{ecx}"(ID) ::: "volatile");
            crate::x86_64::apic::Apic::new(base)
        }
    }
}

pub mod apic {
    const SIVR: usize = 0xF0;

    pub struct Apic {
        base: usize,
    }

    impl Apic {
        pub const fn new(base: usize) -> Apic {
            Apic { base: base }
        }

        pub unsafe fn set_sivr(&self, value: u8) {
            let mut sivr: usize;

            asm!("mov $0, %rax" : "={rax}"(sivr) : "m"(self.base + SIVR) ::: "volatile");
            sivr &= !0xFF;
            sivr |= (value & 0xF0) as usize;
            asm!("mov %rax, $0" :: "{rax}"(sivr), "m"(self.base + SIVR) :: "volatile");
        }
    }
}

pub mod tlb {
    pub unsafe fn flush() {
        let mut value: usize;
        asm!("mov %cr3, %rax" : "={rax}"(value) :::: "volatile"); 
        asm!("mov %rax, %cr3" :: "{rax}"(value) ::: "volatile"); 
    }

    pub unsafe fn update(new_value: usize) {
        asm!("mov %rax, %cr3" :: "{rax}"((new_value & !0xfff)) ::: "volatile");
    }
}

pub mod mmio {
    pub struct Port {
        addr: usize
    }

    impl Port {
        pub fn new(new_addr: usize) -> Port{
            Port { addr: new_addr }
        }

        pub unsafe fn write(&self, value: u8) {
            asm!("outb %al, %dx" :: "{dx}"(self.addr), "{al}"(value) :: "volatile");
        }

        pub unsafe fn read(&self) -> u8 {
            let value: u8;
            asm!("inb %dx, %al" : "={al}"(value) : "{dx}"(self.addr) :: "volatile");
            value
        }
    }
}
