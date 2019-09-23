#![no_std]

use core::panic::PanicInfo;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(multiboot_info: usize) -> ! {
    loop {}
}
