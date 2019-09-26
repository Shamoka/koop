#![no_std]

use core::panic::PanicInfo;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(_multiboot2_info: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    loop {}
}
