use multiboot2::elf;

pub const FLAG_PRESENT: usize = 1 << 0;
pub const FLAG_WRITABLE: usize = 1 << 1;
pub const FLAG_NO_EXEC: usize = 1 << 63;

const ADDR_BITS: usize = 0x000f_ffff_ffff_f000;
const FLAG_BITS: usize = !ADDR_BITS;

#[derive(Debug)]
pub struct Entry {
    pub addr: usize,
    pub flags: usize,
}

impl Entry {
    pub fn new(addr: usize, flags: usize) -> Entry {
        Entry {
            addr: addr & ADDR_BITS,
            flags: flags & FLAG_BITS,
        }
    }

    pub fn from_entry(value: usize) -> Entry {
        Entry {
            addr: value & ADDR_BITS,
            flags: value & FLAG_BITS,
        }
    }

    pub fn from_elf(addr: usize, elf_flags: usize) -> Entry {
        let mut flags = 0;
        if elf_flags & elf::SHF_ALLOC != 0 {
            flags |= FLAG_PRESENT;
        }
        if elf_flags & elf::SHF_WRITE != 0 {
            flags |= FLAG_WRITABLE;
        }
        if elf_flags & elf::SHF_EXECINSTR == 0 {
            flags |= FLAG_NO_EXEC;
        }
        Entry {
            addr: addr & ADDR_BITS,
            flags: flags,
        }
    }

    pub fn unused(&self) -> bool {
        self.addr | self.flags == 0
    }

    pub fn value(&self) -> usize {
        self.addr | self.flags
    }
}
