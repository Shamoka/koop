#![no_std]

use core::panic::PanicInfo;
use mem::allocator::ALLOCATOR;
use vga::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

unsafe fn alloc_test(tab: &mut [*mut u8; 2000], begin: usize, end: usize) {
    for i in begin..end {
        let size = i * 2 + 10;
        tab[i] = ALLOCATOR.memalloc(size);
        if tab[i].is_null() {
            panic!("Alloc number {} failed", i);
        }
        *(tab[i].offset((size - i) as isize - 1)) = 42;
    }
}

unsafe fn dealloc_test(tab: &mut [*mut u8; 2000], begin: usize, end: usize) {
    for i in begin..end {
        ALLOCATOR.memdealloc(tab[i]);
    }
}

unsafe fn test(tab: &mut [*mut u8; 2000]) {
    alloc_test(tab, 0, 2000); // m: 0-2000
    dealloc_test(tab, 500, 1500); // m: 0-500 1500-2000
    alloc_test(tab, 1000, 1500); // m: 0-500 1000-2000
    dealloc_test(tab, 1000, 2000); // m: 0-500
    alloc_test(tab, 500, 2000); // m: 0-2000
    dealloc_test(tab, 0, 2000); // m: None
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        ALLOCATOR.init(multiboot2::Info::new(mb2));
        let mut tab = [0 as *mut u8; 2000];
        for _ in 0..2000 {
            test(&mut tab);
        }
        ALLOCATOR.inspect();
    }
    loop {}
}
