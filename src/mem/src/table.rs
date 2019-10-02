use crate::addr::Addr;
use crate::entry::Entry;
use crate::entry;
use crate::frame;
use crate::AllocError;

pub trait TableLevel {
    type DownLevel: TableLevel;

    fn map_frame(&mut self,
                 addr: &Addr,
                 entry: Entry,
                 frame_allocator: &mut frame::Allocator)
        -> Result<(), AllocError>;
}

macro_rules! table_struct {
    ($T:tt) => {
        #[derive(Copy, Clone)]
        pub struct $T {
            entries: *mut [usize; 512],
            pub base: usize,
            level: usize
        }
    }
}

macro_rules! impl_table {
    ($T:tt, $level: literal) => {
        impl $T{
            pub fn new(addr: &Addr, base: usize) -> $T {
                $T {
                    base: base,
                    entries: addr.addr as *mut [usize; 512],
                    level: $level,
                }
            }

            pub fn flush(&mut self, i: usize, j: usize) {
                for index in i..=j {
                    unsafe {
                        (*self.entries)[index] = 0;
                    }
                }
            }

            pub fn set_entry(&mut self, i: usize, entry: Entry) {
                unsafe {
                    (*self.entries)[i] = entry.value();
                }
            }
        }
    }
}

macro_rules! impl_table_level {
    ($T:tt, $U:tt) => {
        impl TableLevel for $T {
            type DownLevel = $U;

            fn map_frame(&mut self, 
                addr: &Addr,
                entry: Entry,
                frame_allocator: &mut frame::Allocator)
                -> Result<(), AllocError> {
                    let i = addr.get_table_index(self.level);
                    let current_entry = unsafe { Entry::from_entry((*self.entries)[i]) };
                    let mut do_flush = false;
                    if current_entry.unused() {
                        do_flush = true;
                        match frame_allocator.alloc() {
                            Ok(table_frame) => {
                                self.set_entry(i, Entry::new(table_frame.base.addr,
                                        entry::FLAG_WRITABLE
                                        | entry::FLAG_PRESENT));
                            },
                            Err(error) => return Err(error)
                        };
                    }
                    let mut down_level = Self::DownLevel::new(
                        &addr.get_table_addr(self.level - 1, self.base), self.base);
                    if do_flush {
                        down_level.flush(0, 511);
                    }
                    down_level.map_frame(addr, entry, frame_allocator)
                }
        }
    };
    ($T:tt) => {
        impl TableLevel for $T {
            type DownLevel = $T;

            fn map_frame(&mut self, 
                addr: &Addr,
                entry: Entry,
                _frame_allocator: &mut frame::Allocator)
                -> Result<(), AllocError> {
                    let i = addr.get_table_index(self.level);
                    let current_entry = unsafe { Entry::from_entry((*self.entries)[i]) };
                    match current_entry.unused() {
                        true => {
                            self.set_entry(i, entry);
                            Ok(())
                        }
                        false => Err(AllocError::InUse)
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
