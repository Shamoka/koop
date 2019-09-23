#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use vga::println;

#[no_mangle]
pub fn koop() -> ! {
    vga::TEXT_BUFFER.lock().clear();
    println!("Hello {}!", "World");
    println!("Second {}!", "Line");
    loop {}
}
