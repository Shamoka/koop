#![no_std]

use core::panic::PanicInfo;
use mem::allocator::ALLOCATOR;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    println!("OK");
    for block in mem::block::Block::from_memory_bounds(1 << 20, 1 << 32) {
        println!("{:x} {:x}", block.order, block.addr);
    }
    println!("OK");
    loop {}
}
