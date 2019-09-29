#![no_std]

use core::panic::PanicInfo;
use vga::println;
use mem::allocator::ALLOCATOR;
use mem::area;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        if let Err(error) = ALLOCATOR.init(multiboot2::Info::new(mb2)) {
            panic!("Unable to init allocator stage 1 {:?}", error);
        }
    }
    loop {}
}
