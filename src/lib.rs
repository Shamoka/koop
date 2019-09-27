#![no_std]

use core::panic::PanicInfo;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        mem::allocator::ALLOCATOR.lock().init(
            0xffff_ffff_ffff_f000, 
            multiboot2::Info::new(mb2));
    }
    println!("OK");
    loop {}
}
