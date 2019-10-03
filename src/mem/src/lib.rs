#![no_std]
#![feature(asm)]

const UPPER_MEMORY_BOUND: usize = 1 << 20;

mod frame;
mod table;
mod entry;
mod stage1;
mod stage2;
mod memtree;
mod block;
mod stack;

pub mod addr;
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
