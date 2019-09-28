#![no_std]
#![feature(allocator_api)]

const UPPER_MEMORY_BOUND: usize = 1 << 20;

mod addr;
mod bits;
mod frame;
mod table;
mod entry;
mod stack;
mod page_manager;

pub mod area;
pub mod allocator;

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    Uninitialized,
    InUse,
    InvalidAddr,
    Forbidden
}
