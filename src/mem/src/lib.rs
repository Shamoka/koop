#![no_std]
#![feature(allocator_api)]

const UPPER_MEMORY_BOUND: usize = 1 << 20;

mod addr;
mod frame;
mod table;
mod entry;
mod stack;
mod stage1;

pub mod area;
pub mod allocator;

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    Uninitialized,
    InUse,
    InvalidAddr,
    Forbidden,
    InvalidInit
}
