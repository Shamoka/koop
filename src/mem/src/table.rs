use crate::addr::Addr;
use crate::entry;
use crate::entry::Entry;
use crate::frame;

pub trait TableLevel {
    type DownLevel: TableLevel;

    fn map_addr(&mut self, addr: Addr, frame_producer: &mut frame::Allocator);
}

macro_rules! table_struct {
    ($T:tt) => {
        pub struct $T {
            entries: *mut [usize; 512]
        }
    }
}

macro_rules! impl_table {
    ($T:tt) => {
        impl $T{
            pub fn new(addr: Addr) -> $T {
                $T {
                    entries: addr.bits.value as *mut [usize; 512]
                }
            }

            pub fn flush(&mut self) {
                for i in 0..512 {
                    unsafe {
                        (*self.entries)[i] = 0;
                    }
                }
            }
        }
    }
}

macro_rules! impl_table_level {
    ($T:tt, $U: tt, $level:literal) => {
        impl TableLevel for $T {
            type DownLevel = $U;

            fn map_addr(&mut self, addr: Addr, frame_producer: &mut frame::Allocator) {
                let i = addr.get_table_index($level);
                let entry = unsafe { Entry::from_value((*self.entries)[i]) };
                let mut do_flush = false;
                if entry.test_flags(entry::FLAG_WRITABLE | entry::FLAG_PRESENT) == false {
                    let frame = frame_producer.alloc().expect("Out of memory");
                    let new_entry = Entry::new(frame.base, entry::FLAG_WRITABLE | entry::FLAG_PRESENT);
                    do_flush = true;
                    unsafe { (*self.entries)[i] = new_entry.value(); }
                }
                if $level > 1 {
                    let mut down_level = Self::DownLevel::new(addr.get_table_addr($level - 1));
                    if do_flush {
                        down_level.flush();
                    }
                    down_level.map_addr(addr, frame_producer);
                }
            }
        }

    }
}

macro_rules! builder {
    ($T:tt, $U:tt, $level:tt) => {
        table_struct!($T);
        impl_table!($T);
        impl_table_level!($T, $U, $level);
    }
}

builder!(PML4, PDP, 4);
builder!(PDP, PD, 3);
builder!(PD, PT, 2);
builder!(PT, PT, 1);
