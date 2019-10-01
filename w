use crate::pool::{Pool, PoolResult};
use crate::area::Area;

use core::cmp::Ordering;

pub struct MemTree {
    storage: *mut Pool<Node>,
    root: Option<*mut Node>,
    new_node: Option<*mut Node>
}

pub enum InsertResult {
    Done,
    Request(*mut u8),
    Error
}

#[derive(PartialEq)]
enum Color {
    Red,
    Black
}

enum Child {
    Right,
    Left,
    None
}

struct Node {
    content: Area,
    left: Option<*mut Node>,
    right: Option<*mut Node>,
    parent: Option<*mut Node>,
    color: Color
}

impl MemTree {
    pub unsafe fn new(storage: &Area) -> MemTree {
        MemTree {
            storage: Pool::new(storage),
            root: None,
            new_node: None
        }
    }

    pub fn insert(&mut self, new_area: &Area) -> InsertResult {
        unsafe {
            match self.new_node {
                Some(new_node_ptr) => {
                    (*new_node_ptr).init(new_area);
                    match self.root {
                        Some(root_ptr) => {
                            match (*root_ptr).insert(new_node_ptr) {
                                InsertResult::Done => {
                                    (*new_node_ptr).repair();
                                    while let Some(ptr) = (*new_node_ptr).parent() {
                                        self.root = Some(ptr);
                                    }
                                    InsertResult::Done
                                },
                                InsertResult::Error => InsertResult::Error,
                                InsertResult::Request(req) => InsertResult::Request(req)
                            }
                        }
                        None => {
                            self.root = self.new_node.take();
                            InsertResult::Done
                        }
                    }
                },
                None => {
                    match (*self.storage).get_elem() {
                        PoolResult::Done(ptr) => {
                            self.new_node = Some(ptr);
                            self.insert(new_area)
                        },
                        PoolResult::Request(ptr) => InsertResult::Request(ptr as *mut u8),
                        _ => InsertResult::Error
                    }
                }
            }
        }
    }

    pub fn find(&mut self, target: &Area) -> Option<*mut Node> {
        unsafe {
            match self.root {
                Some(root_ptr) => (*root_ptr).find(target),
                None => None
            }
        }
    }

    pub unsafe fn delete(&mut self, target: *mut Node) {
        (*target).delete();
    }
}

impl Node {
    pub fn init(&mut self, area: &Area) {
        self.content = *area;
        self.right = None;
        self.left = None;
        self.parent = None;
        self.color = Color::Red;
    }

    pub unsafe fn delete(&mut self) {
    }

    pub unsafe fn find(&mut self, target: &Area) -> Option<*mut Node> {
        match self.content.cmp(area) {
            Ordering::Less => {
                match self.left {
                    Some(ptr) => (*ptr).find(target),
                    None => None
                }
            },
            Ordering::Greater => {
                match self.right {
                    Some(ptr) => (*ptr).find(target),
                    None => None
                }
            }
            Ordering::Equal => self as *mut Node
        }
    }

    pub unsafe fn insert(&mut self, new_node: *mut Node) -> InsertResult {
        if (*new_node).content.base.addr < self.content.base.addr {
            match self.left {
                Some(node) => return (*node).insert(new_node),
                None => self.left = Some(new_node)
            };
        } else {
            match self.right {
                Some(node) => return (*node).insert(new_node),
                None => self.right = Some(new_node)
            };
        }
        (*new_node).parent = Some(self as *mut Node);
        InsertResult::Done
    }

    pub unsafe fn repair(&mut self) {
        match self.parent {
            Some(parent) => {
                if (*parent).color == Color::Black {
                    self.insert_case_2();
                }
                else if let Some(uncle) = self.uncle() {
                    if (*uncle).color == Color::Red {
                        self.insert_case_3();
                    }
                } else {
                    self.insert_case_4();
                }
            },
            None => self.insert_case_1()
        }
    }

    unsafe fn insert_case_1(&mut self) {
        if let Some(parent) = self.parent {
            (*parent).color = Color::Black;
        }
    }

    unsafe fn insert_case_2(&mut self) {
        return;
    }

    unsafe fn insert_case_3(&mut self) {
        if let Some(parent) = self.parent {
            (*parent).color = Color::Black;
        }
        if let Some(uncle) = self.uncle() {
            (*uncle).color = Color::Black;
        }
        if let Some(grand_parent) = self.grand_parent() {
            (*grand_parent).color = Color::Red;
            (*grand_parent).repair();
        }
    }

    unsafe fn insert_case_4(&mut self) {
        if let Some(parent) = self.parent {
            if let Some(grand_parent) = self.grand_parent() {
                if Some(self as *mut Node) == (*parent).left {
                    (*grand_parent).rotate_right();
                } else {
                    (*grand_parent).rotate_left();
                }
                (*parent).color = Color::Black;
                (*grand_parent).color = Color::Red;
            }
        }
    }

    pub unsafe fn rotate_left(&mut self) {
        match self.right {
            Some(right) => {
                self.right = (*right).left;
                (*right).left = Some(self as *mut Node);
                self.parent = Some(right);
                (*right).parent = Some(self as *mut Node);
                self.rotate_common(right);
            },
            None => ()
        }
    }

    pub unsafe fn rotate_right(&mut self) {
        match self.left {
            Some(left) => {
                self.left = (*left).right;
                (*left).right = Some(self as *mut Node);
                self.parent = Some(left);
                (*left).parent = Some(self as *mut Node);
                self.rotate_common(left);
            },
            None => ()
        }
    }

    pub fn parent(&self) -> Option<*mut Node> {
        self.parent
    }

    unsafe fn rotate_common(&mut self, ptr: *mut Node) {
        if let Some(parent) = self.parent {
            match self.which_child() {
                Child::Right => (*parent).left = Some(ptr),
                Child::Left =>  (*parent).right = Some(ptr),
                Child::None => ()
            }
        }
        (*ptr).parent = self.parent;
    }

    unsafe fn which_child(&mut self) -> Child {
        match self.parent {
            Some(parent) => {
                if let Some(node) = (*parent).left {
                    if node == self as *mut Node {
                        return Child::Left;
                    }
                }
                return Child::Right;
            }
            None => return Child::None
        }
    }

    unsafe fn grand_parent(&self) -> Option<*mut Node> {
        match self.parent {
            Some(parent) => (*parent).parent,
            None => None
        }
    }

    unsafe fn sibling(&mut self) -> Option<*mut Node> {
        match self.parent {
            Some(parent) => {
                if let Some(left) = (*parent).left {
                    if left == self as *mut Node {
                        return (*parent).right;
                    }
                };
                if let Some(right) = (*parent).right {
                    if right == self as *mut Node {
                        return (*parent).left;
                    }
                };
                None
            },
            None => None
        }
    }

    unsafe fn uncle(&mut self) -> Option<*mut Node> {
        match self.parent {
            Some(parent) => (*parent).sibling(),
            None => None
        }
    }
}
