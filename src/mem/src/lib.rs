#![no_std]
#![feature(allocator_api)]

pub const UPPER_MEMORY_BOUND: usize = 1 << 20;

pub mod addr;
pub mod bits;
pub mod frame;
pub mod table;
pub mod entry;
pub mod area;
pub mod allocator;
pub mod stack;

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    Uninitialized,
    InUse,
    InvalidAddr,
    Forbidden
}
