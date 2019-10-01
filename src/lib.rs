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
    loop {}
}
