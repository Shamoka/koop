#![no_std]

use core::panic::PanicInfo;
use vga::println;
use mem::allocator::TMP_ALLOCATOR;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        TMP_ALLOCATOR.lock().init(
            0xffff_ffff_ffff_f000, 
            multiboot2::Info::new(mb2));
    }
    let area = mem::area::Area::new(0o003_123_234_345 << 12, 0xfff000, mem::area::Alignment::Page);
    match TMP_ALLOCATOR.lock().map_area(&area) {
        Ok(_) => (),
        Err(error) => panic!("{:?}", error)
    };
    unsafe {
        *(0o003_123_234_350 as *mut u8) = 1u8;
    }
    println!("OK");
    loop {}
}
