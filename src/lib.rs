#![no_std]

use core::panic::PanicInfo;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(multiboot2_info: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    let mb2 = multiboot2::Info::new(multiboot2_info);
    let _frame_allocator = mem::frame::Allocator::new(&mb2);
    loop {}
}
