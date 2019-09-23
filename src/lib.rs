#![no_std]

use core::panic::PanicInfo;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop() -> ! {
    vga::TEXT_BUFFER.lock().clear();
    println!("Hello {}!", "World");
    loop {}
}
