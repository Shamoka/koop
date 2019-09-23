#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use vga;

#[no_mangle]
pub fn koop() -> ! {
    let mut v = vga::TEXT_BUFFER.lock();

    v.clear();
    v.write("Hello World!\n", vga::Color::White, vga::Color::Black);
    v.write("Second line\n", vga::Color::White, vga::Color::Black);
    loop {}
}
