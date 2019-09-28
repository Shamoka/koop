#![no_std]

use core::panic::PanicInfo;
use vga::println;
use mem::allocator::ALLOCATOR;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        if let Err(error) = ALLOCATOR.lock().stage0(multiboot2::Info::new(mb2)) {
            panic!("{:?}", error);
        }
    }
    let area = mem::area::Area::new(0o003_123_234_345 << 12, 0x1000000, mem::area::Alignment::Page);
    for page in area.pages() {
        let new_area = mem::area::Area::new(page.bits.value, 0x1000, mem::area::Alignment::Page);
        match ALLOCATOR.lock().memmap(&new_area) {
            Ok(_) => (),
            Err(error) => panic!("{:?}", error)
        };
    }
    println!("OK");
    loop {}
}
