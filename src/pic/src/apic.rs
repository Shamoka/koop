const SPURIOUS: u8 = 0xff;

pub unsafe fn init() {

    asm::x86_64::apic::disable_pic();
    let apic = asm::x86_64::reg::apic_base::enable();
    apic.set_sivr(SPURIOUS);
}
