#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

mod entry;
pub mod handlers;

use crate::entry::Entry;

use spinlock::Mutex;

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};

pub static IDT: IDT_ = IDT_::new();

pub struct StackFrame {
    pub ip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub sp: u64,
    pub ss: u64,
}

type HandlerFunc = extern "x86-interrupt" fn(&mut StackFrame);
type HandlerFuncError = extern "x86-interrupt" fn(&mut StackFrame, usize);
type IDTPtr = *const [Entry; 256];

#[repr(C)]
pub struct IDT_ {
    entries: UnsafeCell<[Entry; 256]>,
    mutex: Mutex<()>,
}

#[repr(C, packed)]
struct IDTR {
    size: u16,
    ptr: usize,
}

impl IDT_ {
    pub const fn new() -> IDT_ {
        IDT_ {
            entries: UnsafeCell::new([Entry::new_empty(); 256]),
            mutex: Mutex::new(()),
        }
    }

    pub fn init(&self) {
        let _lock = self.mutex.lock();
        unsafe {
            self.set_handler(0x80, handlers::syscall);
            self.set_handler_with_error(0xd, handlers::general_protection_fault);
            self.set_handler_with_error(0xe, handlers::page_fault);
            self.set_handler_with_error(0x8, handlers::double_fault);
            self.load(self.entries.get());
        }
    }

    unsafe fn load(&self, ptr: IDTPtr) {
        let idt_r = IDTR {
            size: 4096 - 1,
            ptr: ptr as usize,
        };
        asm!("lidt ($0)" :: "r" (&idt_r as *const IDTR));
    }

    unsafe fn set_handler(&self, index: usize, handler: HandlerFunc) {
        let entries = self.entries.get();
        (*entries)[index].set_addr(handler as usize);
        (*entries)[index].set_present(true);
        (*entries)[index].set_cs();
    }

    unsafe fn set_handler_with_error(&self, index: usize, handler: HandlerFuncError) {
        let entries = self.entries.get();
        (*entries)[index].set_addr(handler as usize);
        (*entries)[index].set_present(true);
        (*entries)[index].set_cs();
    }
}

unsafe impl Send for IDT_ {}
unsafe impl Sync for IDT_ {}
