use crate::header::Header;

use core::mem::size_of;

#[repr(C, packed)]
pub struct MADT {
    pub header: Header,
    pub local_apic_address: u32,
    pub flags: u32
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
pub struct MADT_LAPIC {
    pub header: EntryHeader,
    pub proc_id: u8,
    pub apic_id: u8,
    pub flags: u32
}

#[repr(C, packed)]
pub struct MADT_IOAPIC {
    pub header: EntryHeader,
    pub ioapic_id: u8,
    pub _res: u8,
    pub ioapic_addr: u32,
    pub global_system_interrupt_base: u32
}

#[repr(C, packed)]
pub struct MADT_ISO {
    pub header: EntryHeader,
    pub bus_source: u8,
    pub irq_source: u8,
    pub global_system_interrupt: u32,
    pub flags: u16
}

#[repr(C, packed)]
pub struct MADT_NMI {
    pub header: EntryHeader,
    pub proc_id: u8,
    pub flags: u16,
    pub lint: u8
}

#[repr(C, packed)]
pub struct MADT_LAPICOverride {
    pub header: Header,
    _res: u16,
    pub addr: u64
}

pub enum Entry {
    EntryLAPIC(*const MADT_LAPIC),
    EntryIOAPIC(*const MADT_IOAPIC),
    EntryISO(*const MADT_ISO),
    EntryNMI(*const MADT_NMI),
    EntryLAPICOverride(*const MADT_LAPICOverride),
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
                0 => Some(Entry::EntryLAPIC(entry_ptr as *const MADT_LAPIC)),
                1 => Some(Entry::EntryIOAPIC(entry_ptr as *const MADT_IOAPIC)),
                2 => Some(Entry::EntryISO(entry_ptr as *const MADT_ISO)),
                4 => Some(Entry::EntryNMI(entry_ptr as *const MADT_NMI)),
                5 => Some(Entry::EntryLAPICOverride(entry_ptr as *const MADT_LAPICOverride)),
                _ => Some(Entry::EntryUnknown)
            };
            self.pos += (*entry_ptr).length as usize;
            entry
        }
    }
}
