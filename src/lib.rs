#![no_std]
#![feature(alloc_error_handler)]

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
        loop {
            asm::x86_64::instruction::hlt();
        }
    }
}

#[no_mangle]
pub fn koop(mb2: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    unsafe {
        let mb2_info = multiboot2::Info::new(mb2);
        let rdsp = mb2_info
            .get_rsdp()
            .expect("No RSDP found in multiboot2 info")
            .addr();
        ALLOCATOR.init(&mb2_info);
        ALLOCATOR
            .id_map(rdsp, mem::frame::FRAME_SIZE)
            .expect("Unable to map RDST");
        pic::init(rdsp);
        loop {
            asm::x86_64::instruction::hlt();
        }
    }
}
