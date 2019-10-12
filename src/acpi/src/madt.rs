use crate::header::Header;

use core::mem::size_of;

#[repr(C, packed)]
pub struct MADT {
    header: Header,
    local_apic_address: u32,
    flags: u32
}

pub struct MADTIter<'a> {
    madt: &'a MADT,
    pos: usize
}

#[repr(C, packed)]
pub struct EntryHeader {
    pub entry_type: u8,
    pub length: u8
}

#[repr(C, packed)]
pub struct LAPIC {
    header: EntryHeader,
    proc_id: u8,
    apic_id: u8,
    flags: u32
}

#[repr(C, packed)]
pub struct IOAPIC {
    header: EntryHeader,
    ioapic_id: u8,
    _res: u8,
    ioapic_addr: u32,
    global_system_interrupt_base: u32
}

#[repr(C, packed)]
pub struct ISO {
    header: EntryHeader,
    bus_source: u8,
    irq_source: u8,
    global_system_interrupt: u32,
    flags: u16
}

#[repr(C, packed)]
pub struct NMI {
    header: EntryHeader,
    proc_id: u8,
    flags: u16,
    lint: u8
}

#[repr(C, packed)]
pub struct LAPICOverride {
    header: Header,
    _res: u16,
    addr: u64
}

pub enum Entry {
    EntryLAPIC(*const LAPIC),
    EntryIOAPIC(*const IOAPIC),
    EntryISO(*const ISO),
    EntryNMI(*const NMI),
    EntryLAPICOverride(*const LAPICOverride),
    EntryUnknown
}

impl MADT {
    pub unsafe fn entries(&self) -> MADTIter {
        MADTIter {
            madt: self,
            pos: self as *const MADT as usize + size_of::<MADT>(),
        }
    }
}

impl<'a> Iterator for MADTIter<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos - self.madt as *const MADT as usize >= self.madt.header.length as usize {
            return None
        }
        let entry_ptr = self.pos as *const EntryHeader;
        unsafe {
            let entry = match (*entry_ptr).entry_type {
                0 => Some(Entry::EntryLAPIC(entry_ptr as *const LAPIC)),
                1 => Some(Entry::EntryIOAPIC(entry_ptr as *const IOAPIC)),
                2 => Some(Entry::EntryISO(entry_ptr as *const ISO)),
                4 => Some(Entry::EntryNMI(entry_ptr as *const NMI)),
                5 => Some(Entry::EntryLAPICOverride(entry_ptr as *const LAPICOverride)),
                _ => Some(Entry::EntryUnknown)
            };
            self.pos += (*entry_ptr).length as usize;
            entry
        }
    }
}
