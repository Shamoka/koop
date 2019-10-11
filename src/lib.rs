#![no_std]
#![feature(alloc_error_handler)]

use idt::IDT;
use mem::allocator::ALLOCATOR;
use vga::println;

extern crate alloc;

use core::alloc::Layout;
use core::panic::PanicInfo;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!(
        "Cannot allocate {} bytes aligned to {}",
        layout.size(),
        layout.align()
    );
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    unsafe {
        asm::x86_64::instruction::hlt();
    }
    loop {}
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        ALLOCATOR.init(multiboot2::Info::new(mb2));
        IDT.init();
        pic::init(true);
        asm::x86_64::instruction::hlt();
    }
    loop {}
}
