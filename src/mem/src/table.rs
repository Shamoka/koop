use crate::addr::Addr;
use crate::entry;
use crate::entry::Entry;
use crate::frame;

pub trait TableLevel {
    type DownLevel: TableLevel;

    fn map_addr(&mut self, addr: Addr, frame_allocator: &mut frame::Allocator);
}

macro_rules! table_struct {
    ($T:tt) => {
        pub struct $T {
            entries: *mut [usize; 512],
            level: usize
        }
    }
}

macro_rules! impl_table {
    ($T:tt, $level: literal) => {
        impl $T{
            pub fn new(addr: Addr) -> $T {
                $T {
                    entries: addr.bits.value as *mut [usize; 512],
                    level: $level
                }
            }

            pub fn flush(&mut self) {
                for i in 0..512 {
                    unsafe {
                        (*self.entries)[i] = 0;
                    }
                }
            }

            pub fn set_entry(&mut self, i: usize, frame_allocator: &mut frame::Allocator) {
                let frame = frame_allocator.alloc().expect("Out of memory");
                let new_entry = Entry::new(frame.base, entry::FLAG_WRITABLE | entry::FLAG_PRESENT);
                unsafe {
                    (*self.entries)[i] = new_entry.value();
                }
            }
        }
    }
}

macro_rules! impl_table_level {
    ($T:tt, $U:tt) => {
        impl TableLevel for $T {
            type DownLevel = $U;

            fn map_addr(&mut self, addr: Addr, frame_allocator: &mut frame::Allocator) {
                let i = addr.get_table_index(self.level);
                let entry = unsafe { Entry::from_value((*self.entries)[i]) };
                let mut do_flush = false;
                if entry.value() == 0 {
                    do_flush = true;
                    self.set_entry(i, frame_allocator);
                }
                let mut down_level = Self::DownLevel::new(addr.get_table_addr(self.level - 1));
                if do_flush {
                    down_level.flush();
                }
                down_level.map_addr(addr, frame_allocator);
            }
        }
    };
    ($T:tt) => {
        impl TableLevel for $T {
            type DownLevel = $T;

            fn map_addr(&mut self, addr: Addr, frame_allocator: &mut frame::Allocator) {
                let i = addr.get_table_index(self.level);
                let entry = unsafe { Entry::from_value((*self.entries)[i]) };
                if entry.value() == 0 {
                    self.set_entry(i, frame_allocator);
                }
            }
        }
    };
}

macro_rules! builder {
    ($T:tt, $U:tt, $level:tt) => {
        table_struct!($T);
        impl_table!($T, $level);
        impl_table_level!($T, $U);
    };
    ($T:tt) => {
        table_struct!($T);
        impl_table!($T, 1);
        impl_table_level!($T);
    }
}

builder!(PML4, PDP, 4);
builder!(PDP, PD, 3);
builder!(PD, PT, 2);
builder!(PT);
