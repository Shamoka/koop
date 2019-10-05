use crate::stage2;
use crate::addr::Addr;

use spinlock::Mutex;

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};
use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

pub const PML4_ADDR: Addr = Addr::new(0xffff_ffff_ffff_f000);

pub struct Allocator<'a> {
    internal: UnsafeCell<Stage<'a>>,
    mutex: Mutex<()>
}

pub enum Stage<'a> {
    Stage0,
    Stage2(stage2::Allocator<'a>)
}

impl<'a> Allocator<'a> {
    pub const fn new() -> Allocator<'a> {
        Allocator {
            internal: UnsafeCell::new(Stage::Stage0),
            mutex: Mutex::new(())
        }
    }

    pub unsafe fn init(&self, mb2: multiboot2::Info) {
        let _lock = self.mutex.lock();
        *self.internal.get() = Stage::Stage2(stage2::Allocator::new(mb2));
    }

    pub unsafe fn memalloc(&self, len: usize) -> *mut u8 {
        let _lock = self.mutex.lock();
        match &mut *self.internal.get() {
            Stage::Stage2(allocator) => allocator.alloc(len),
            _ => 0 as *mut u8
        }
    }

    pub unsafe fn inspect(&self) {
        let _lock = self.mutex.lock();
        match &mut *self.internal.get() {
            Stage::Stage2(allocator) => allocator.inspect(),
            _ => {}
        }
    }

    pub unsafe fn memdealloc(&self, ptr: *mut u8) {
        let _lock = self.mutex.lock();
        if let Stage::Stage2(allocator) = &mut *self.internal.get() {
            allocator.dealloc(ptr);
        }
    }
}

unsafe impl<'a> Send for Allocator<'a> {}
unsafe impl<'a> Sync for Allocator<'a> {}

unsafe impl<'a> GlobalAlloc for Allocator<'a> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        0 as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    }
}
