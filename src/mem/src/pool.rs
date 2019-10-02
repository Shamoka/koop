use crate::area::Area;

use core::mem::size_of;

pub struct Pool<T: Sized> {
    root: Option<*mut PoolNode<T>>,
    storage: *mut PoolNode<T>,
    size: usize,
    in_storage: usize,
    request: Option<*mut Pool<T>>,
    pos: usize
}

pub struct PoolNode<T: Sized> {
    content: T,
    next: Option<*mut PoolNode<T>>
}

pub enum PoolResult<T: Sized> {
    Done(*mut T),
    Request(*mut Pool<T>),
    Denied,
    Accepted
}

impl<T: Sized> Pool<T> {
    pub unsafe fn new(pool_area: &Area, alloc_area: &Area) -> *mut Pool<T> {
        let mut pool_ptr = pool_area.base.addr as *mut Pool<T>;
        (*pool_ptr).root = None;
        (*pool_ptr).storage = alloc_area.base.addr as *mut PoolNode<T>;
        (*pool_ptr).size = alloc_area.len;
        (*pool_ptr).in_storage = alloc_area.len / size_of::<PoolNode<T>>();
        (*pool_ptr).request = None;
        (*pool_ptr).pos = 0;
        pool_ptr
    }

    pub unsafe fn get_elem(&mut self) -> PoolResult<T> {
        match self.root {
            Some(node) => {
                let ret = (*node).content_ptr();
                self.root = (*node).next;
                PoolResult::Done(ret)
            },
            None => {
                match self.pos < self.in_storage {
                    false => self.get_page_request(),
                    true => {
                        let new_node = self.storage.offset(self.pos as isize);
                        let ret = (*new_node).content_ptr();
                        self.pos += 1;
                        PoolResult::Done(ret)
                    }
                }
            }
        }
    }

    pub unsafe fn give_elem(&mut self, elem: *mut PoolNode<T>) {
        (*elem).next = self.root;
        self.root = Some(elem);
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
        self.in_storage += size / size_of::<PoolNode<T>>();
    }
}

impl<T: Sized> PoolNode<T> {
    pub fn content_ptr(&mut self) -> *mut T {
        &mut self.content as *mut T
    }
}
