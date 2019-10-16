#![no_std]

use core::cell::UnsafeCell;
use core::marker::Sized;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Mutex<T: Sized> {
    lock: AtomicBool,
    content: UnsafeCell<T>,
}

pub struct Guard<'a, T: Sized> {
    lock: &'a AtomicBool,
    content: &'a mut T,
}

impl<T: Sized> Mutex<T> {
    pub const fn new(value: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            content: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) {}
        Guard {
            lock: &self.lock,
            content: unsafe { &mut *self.content.get() },
        }
    }
}

impl<'a, T: Sized> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<'a, T: Sized> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<'a, T: Sized> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.content
    }
}

unsafe impl<T: Sized> Send for Mutex<T> {}
unsafe impl<T: Sized> Sync for Mutex<T> {}
