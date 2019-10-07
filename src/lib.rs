#![no_std]

use core::panic::PanicInfo;
use mem::allocator::ALLOCATOR;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

unsafe fn alloc_test(tab: &mut [*mut u8; 2000]) {
    for i in 0..2000 {
        let size = i * 2 + 10;
        tab[i] = ALLOCATOR.memalloc(size);
        if tab[i].is_null() {
            panic!("Alloc number {} failed", i);
        }
        *(tab[i] as *mut u8) = 42;
    }
    for i in 0..500 {
        ALLOCATOR.memdealloc(tab[i]);
    }
    for i in 0..500 {
        let size = i * 2 + 10;
        tab[i] = ALLOCATOR.memalloc(size);
        if tab[i].is_null() {
            panic!("Alloc number {} failed", i);
        }
        *(tab[i] as *mut u8) = 42;
    }
    for i in 0..2000 {
        ALLOCATOR.memdealloc(tab[i]);
    }
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        ALLOCATOR.init(multiboot2::Info::new(mb2));
        let mut tab = [0 as *mut u8; 2000];
        for _ in 0..10000 {
            alloc_test(&mut tab);
        }
        ALLOCATOR.inspect();
    }
    loop {}
}
