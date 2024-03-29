pub mod instruction {
    pub unsafe fn int_80() {
        asm!("int $0" :: "N"(0x80) :::);
    }

    pub unsafe fn hlt() {
        asm!("hlt");
    }
}

pub mod reg {
    pub mod efer {
        const ID: usize = 0xC0000080;

        pub const BIT_NXE: usize = 11;

        pub unsafe fn set_bit(bit: usize) {
            let rax: usize;
            let rdx: usize;
            asm!("rdmsr" : "={rdx}"(rdx), "={rax}"(rax) : "{rcx}"(ID) :::"volatile");
            asm!("wrmsr" :: "{rdx}"(rdx), "{rax}"(rax | 1 << bit), "{rcx}"(ID) ::: "volatile");
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
