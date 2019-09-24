#![no_std]

use core::panic::PanicInfo;
use vga::println;
use multiboot2;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub fn koop(multiboot2_info: usize) -> ! {
    vga::TEXT_BUFFER.lock().clear();
    let mb2 = multiboot2::Info::new(multiboot2_info);
    match mb2.get_elf_sections() {
        Some(sections) => {
            for section in sections {
                println!("0x{:x} 0x{:x} 0x{:x}", section.sh_addr, section.sh_size, section.sh_flags);
            }
        },
        None => ()
    }
    loop {}
}
