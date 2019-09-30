use crate::area::Area;

use core::mem::size_of;

pub struct Pool<T: Sized> {
    root: Option<*mut Node<T>>,
    storage: *mut Node<T>,
    size: usize,
    in_storage: usize,
    request: Option<*mut Pool<T>>
}

struct Node<T: Sized> {
    content: T,
    next: Option<*mut Node<T>>
}

pub enum PoolResult<T: Sized> {
    Done(*mut T),
    Request(*mut Pool<T>),
    Denied,
    Accepted
}

impl<T: Sized> Pool<T> {
    pub unsafe fn new(area: &Area) -> *mut Pool<T> {
        let mut pool_ptr = area.base.addr as *mut Pool<T>;
        (*pool_ptr).root = None;
        (*pool_ptr).storage = (area.base.addr + size_of::<Pool<T>>()) as *mut Node<T>;
        (*pool_ptr).size = area.len - size_of::<Pool<T>>();
        (*pool_ptr).in_storage = (*pool_ptr).size / size_of::<Node<T>>();
        pool_ptr
    }

    pub unsafe fn get_elem(&mut self) -> PoolResult<T> {
        match self.root {
            Some(node) => {
                let ret = &mut (*node).content as *mut T;
                self.root = (*node).next;
                PoolResult::Done(ret)
            },
            None => {
                match self.in_storage {
                    0 => self.get_page_request(),
                    _ => {
                        self.in_storage -= 1;
                        let mut new_node = self.storage.offset(self.in_storage as isize);
                        (*new_node).next = self.root;
                        self.root = Some(new_node);
                        PoolResult::Done(&mut (*new_node).content as *mut T)
                    }
                }
            }
        }
    }

    pub fn fulfill_request(&mut self, area: &Area) -> PoolResult<T> {
        match self.request {
            Some(request) => {
                match area.base.addr as *mut Pool<T> == request {
                    true => {
                        self.grow(area.len);
                        PoolResult::Accepted
                    }
                    false => PoolResult::Denied
                }
            },
            None => PoolResult::Denied
        }
    }

    pub fn get_page_request(&mut self) -> PoolResult<T> {
        let next_page = (self as *mut Pool<T>) as usize + self.size;
        self.request = Some(next_page as *mut Pool<T>);
        PoolResult::Request(next_page as *mut Pool<T>)
    }

    fn grow(&mut self, size: usize) {
        self.size += size;
        self.in_storage += size / size_of::<Node<T>>();
    }
}
