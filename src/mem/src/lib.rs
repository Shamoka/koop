#![no_std]

pub const UPPER_MEMORY_BOUND: usize = 1 << 20;

pub enum Error {
    EOM
}

pub mod frame;
pub mod addr;
pub mod bits;
pub mod area;
