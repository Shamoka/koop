#![no_std]
#![feature(asm)]

const UPPER_MEMORY_BOUND: usize = 1 << 20;

mod entry;
mod memtree;
mod slab;
mod stack;
mod stage1;
mod stage2;
mod table;

pub mod addr;
pub mod allocator;
pub mod area;
pub mod block;
pub mod frame;

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    Uninitialized,
    InUse,
    InvalidAddr,
    Forbidden,
    InvalidInit,
}
