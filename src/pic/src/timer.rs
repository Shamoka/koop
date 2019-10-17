enum TimerMode {
    ONE_SHOT,
    PERIODIC,
    TSC_DEADLINE,
}

pub struct Timer {
    mode: TimerMode,
}

impl Timer {
    pub unsafe fn new() -> Timer {
        if asm::x86_64::instruction::cpuid::check_tsc_deadline() {
            vga::println!("TSC");
            Timer {
                mode: TimerMode::TSC_DEADLINE,
            }
        } else {
            vga::println!("No TSC");
            Timer {
                mode: TimerMode::ONE_SHOT,
            }
        }
    }
}
