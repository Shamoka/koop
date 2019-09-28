use crate::addr::Addr;
use crate::entry;
use crate::entry::Entry;
use crate::frame;
use crate::AllocError;

pub trait TableLevel {
    type DownLevel: TableLevel;

    fn map_addr(&mut self,
                addr: &Addr,
                frame_allocator: &mut frame::Allocator)
        -> Result<(), AllocError>;
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
            pub fn new(addr: &Addr) -> $T {
                $T {
                    entries: addr.bits.value as *mut [usize; 512],
                    level: $level
                }
            }

            pub fn flush(&mut self, i: usize, j: usize) {
                for index in i..=j {
                    unsafe {
                        (*self.entries)[index] = 0;
                    }
                }
            }

            pub fn set_entry(&mut self,
                             i: usize, 
                             frame_allocator: &mut frame::Allocator)
                -> Result<(), AllocError> {
                    match frame_allocator.alloc() {
                        Some(frame) => {
                            let new_entry = Entry::new(frame.base, entry::FLAG_WRITABLE | entry::FLAG_PRESENT);
                            unsafe {
                                (*self.entries)[i] = new_entry.value();
                            }
                            Ok(())
                        },
                        None => Err(AllocError::OutOfMemory)
                    }
                }
        }
    }
}

macro_rules! impl_table_level {
    ($T:tt, $U:tt) => {
        impl TableLevel for $T {
            type DownLevel = $U;

            fn map_addr(&mut self, 
                        addr: &Addr,
                        frame_allocator: &mut frame::Allocator)
                -> Result<(), AllocError> {
                    let i = addr.get_table_index(self.level);
                    let entry = unsafe { Entry::from_value((*self.entries)[i]) };
                    let mut do_flush = false;
                    if entry.value() == 0 {
                        do_flush = true;
                        if let Err(some_error) = self.set_entry(i, frame_allocator) {
                            return Err(some_error);
                        }
                    }
                    let mut down_level = Self::DownLevel::new(&addr.get_table_addr(self.level - 1));
                    if do_flush {
                        down_level.flush(0, 511);
                    }
                    down_level.map_addr(addr, frame_allocator)
                }
        }
    };
    ($T:tt) => {
        impl TableLevel for $T {
            type DownLevel = $T;

            fn map_addr(&mut self, 
                        addr: &Addr,
                        frame_allocator: &mut frame::Allocator)
                -> Result<(), AllocError> {
                    let i = addr.get_table_index(self.level);
                    let entry = unsafe { Entry::from_value((*self.entries)[i]) };
                    match entry.value() {
                        0 => self.set_entry(i, frame_allocator),
                        _ => Err(AllocError::InUse)
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
