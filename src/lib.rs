#![no_std]

use core::panic::PanicInfo;
use vga::println;
use serial;
use spinlock;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop() -> ! {
    unsafe {
        let serial = spinlock::Mutex::new(serial::Port::new(serial::ComAddr::Com1));
        serial.lock().write_str("Hello World!");
    }
    loop {}
}
