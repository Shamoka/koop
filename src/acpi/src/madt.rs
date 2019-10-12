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
pub struct LocalAPIC {
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

pub enum Entry {
    EntryLocalAPIC(*const LocalAPIC),
    EntryIOAPIC(*const IOAPIC),
    EntryUnimplemented
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
                0 => Some(Entry::EntryLocalAPIC(entry_ptr as *const LocalAPIC)),
                1 => Some(Entry::EntryIOAPIC(entry_ptr as *const IOAPIC)),
                _ => Some(Entry::EntryUnimplemented)
            };
            self.pos += (*entry_ptr).length as usize;
            entry
        }
    }
}
