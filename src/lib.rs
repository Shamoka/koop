#![no_std]

use core::panic::PanicInfo;
use vga::println;

use mem::table::TableLevel;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(multiboot2_info: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    let target = mem::addr::Addr::new(0o177777_746_123_345_456_0123);
    let mb2 = multiboot2::Info::new(multiboot2_info);
    let mut frame_allocator = mem::frame::Allocator::new(&mb2);
    let pml4 = mem::addr::Addr::new(0xffff_ffff_ffff_f000);
    let mut pml4_table = mem::table::PML4::new(pml4);
    pml4_table.map_addr(target, &mut frame_allocator);
    unsafe {
        *(0o177777_746_123_345_456_0123 as *mut u8) = 1;
    }
    println!("OK");
    loop {}
}
