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
        ALLOCATOR.init(multiboot2::Info::new(mb2));
        let mut ptrs = [0 as *mut u8; 1000];
        for i in 0..1000 {
            let size = i * 2 + 10;
            ptrs[i] = ALLOCATOR.memalloc(size);
            if ptrs[i].is_null() {
                panic!("Alloc number {} failed", i);
            }
            for j in 0..size {
                (*ptrs[i].offset(j as isize)) = 42;
            }
        }
        for i in 0..1000 {
            ALLOCATOR.memdealloc(ptrs[i]);
        }
        ALLOCATOR.inspect();
    }
    println!("OK");
    loop {}
}
