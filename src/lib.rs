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
    let addr = mem::addr::Addr::new(0o177777_752_123_546_765_1234);
    for table in addr.tables() {
        vga::println!("{:o}", table.bits.value);
    }
    loop {}
}
