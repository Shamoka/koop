#![no_std]

use core::panic::PanicInfo;
use mem::allocator::ALLOCATOR;
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
        if let Err(error) = ALLOCATOR.init(multiboot2::Info::new(mb2)) {
            panic!("{:?}", error);
        }
        let mut ptrs = [0 as *mut u8; 100];
        for i in 1..100 {
            ptrs[i] = ALLOCATOR.memalloc(i * 20 + 10);
            if ptrs[i].is_null() {
                panic!("Alloc number {} failed", i);
            }
        }
        for i in 0..10 {
            ALLOCATOR.memdealloc(ptrs[i]);
        }
        ALLOCATOR.inspect();
    }
    println!("OK");
    loop {}
}
