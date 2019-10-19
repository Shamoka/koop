use crate::StackFrame;

fn dump_int_stack_frame(sf: &StackFrame) {
    vga::println!("ip: {:x}", sf.ip);
    vga::println!("cs: {:x}", sf.ip);
    vga::println!("rflags: {:b}", sf.rflags);
    vga::println!("sp: {:x}", sf.sp);
    vga::println!("ss: {:x}", sf.ss);
}

pub extern "x86-interrupt" fn syscall(_sf: &mut StackFrame) {
    vga::println!("Syscall handler");
}

pub extern "x86-interrupt" fn page_fault(sf: &mut StackFrame, err: usize) {
    vga::println!("Page fault in kernel. Stopping exection");
    dump_int_stack_frame(sf);
    vga::println!("error code: {:x}", err);
    unsafe {
        asm::x86_64::instruction::hlt();
    }
}

pub extern "x86-interrupt" fn general_protection_fault(sf: &mut StackFrame, err: usize) {
    vga::println!("General protection fault in kernel. Stopping exection");
    dump_int_stack_frame(sf);
    vga::println!("error code: {:x}", err);
    unsafe {
        asm::x86_64::instruction::hlt();
    }
}

pub extern "x86-interrupt" fn double_fault(sf: &mut StackFrame, err: usize) {
    vga::println!("Double fault in kernel. Stopping execution");
    dump_int_stack_frame(sf);
    vga::println!("error code: {:x}", err);
    unsafe {
        asm::x86_64::instruction::hlt();
    }
}

pub extern "x86-interrupt" fn timer(_sf: &mut StackFrame) {
    vga::println!("timer");
}
