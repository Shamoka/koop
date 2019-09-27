#![no_std]

pub const UPPER_MEMORY_BOUND: usize = 1 << 20;

pub mod addr;
pub mod bits;
pub mod frame;
pub mod page;
pub mod table;
pub mod entry;
pub mod area;
