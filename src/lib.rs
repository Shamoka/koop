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
        let area = area::Area::new(0o123_234_345_456_0000, 0xf000, mem::addr::AddrType::Virtual);
        if let Err(error) = ALLOCATOR.memmap(&area) {
            panic!("Allocation error {:?}", error);
        }
        *((area.base.addr + 42usize) as *mut u8) = 42;
    }
    println!("OK");
    loop {}
}
