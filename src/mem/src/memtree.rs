use crate::block::Block;
use crate::area::Area;

#[derive(Copy, Clone)]
pub struct MemTree {
    pub root: Option<MemTreeNode>
}

pub enum InsertResult {
    Done,
    Request,
    Error
}

#[derive(Copy, Clone, PartialEq)]
enum Color {
    Red,
    Black
}

enum Child {
    Right,
    Left,
    None
}

#[derive(Copy, Clone)]
pub struct MemTreeNode {
    pub content: Block,
    pub left: Option<*mut MemTreeNode>,
    right: Option<*mut MemTreeNode>,
    parent: Option<*mut MemTreeNode>,
    color: Color
}

impl MemTree {
    pub const fn new() -> MemTree {
        MemTree { root: None }
    }

    pub fn new_init(block: &Block) -> MemTree {
        MemTree { root: Some(MemTreeNode::new(block)) }
    }

    pub fn take(&mut self) -> (Option<Block>, Option<*mut MemTreeNode>) {
        unsafe {
            match self.root {
                Some(ref mut root) => {
                    if root.left.is_none() && root.right.is_none() {
                        match self.root.take() {
                            Some(node) => return (Some(node.content), None),
                            None => return (None, None)
                        }
                    }
                    match root.left {
                        Some(left) => {
                            let content = (*left).content;
                            (*left).remove();
                            return (Some(content), Some(left));
                        }
                        None => ()
                    };
                    match root.right {
                        Some(right) => {
                            let content = (*right).content;
                            (*right).remove();
                            return (Some(content), Some(right));
                        }
                        None => ()
                    };
                    return (None, None);
                },
                None => (None, None)
            }
        }
    }

    pub fn insert(&mut self, new_node: *mut MemTreeNode) -> InsertResult {
        unsafe {
            match self.root {
                Some(mut root) => {
                    match root.insert(new_node) {
                        InsertResult::Done => {
                            (*new_node).repair();
                            while let Some(ptr) = (*new_node).parent() {
                                self.root = Some(*ptr);
                            }
                            InsertResult::Done
                        },
                        InsertResult::Error => InsertResult::Error,
                        InsertResult::Request => InsertResult::Request
                    }
                }
                None => {
                    self.root = Some(*new_node);
                    InsertResult::Done
                }
            }
        }
    }

    pub fn find(&mut self, target: &Area) -> Option<*mut MemTreeNode> {
        unsafe {
            match self.root {
                Some(mut root) => root.find(target),
                None => None
            }
        }
    }
}

impl MemTreeNode {
    pub fn new(block: &Block) -> MemTreeNode {
        MemTreeNode {
            content: *block,
            left: None,
            right: None,
            parent: None,
            color: Color::Red
        }
    }

    pub unsafe fn remove(&mut self) {
    }

    pub unsafe fn find(&mut self, target: &Area) -> Option<*mut MemTreeNode> {
        if target.base.addr == self.content.addr {
            return Some(self as *mut MemTreeNode);
        }
        if target.base.addr < self.content.addr {
            match self.left {
                Some(ptr) => return (*ptr).find(target),
                None => return None
            }
        } else if target.base.addr > self.content.addr {
            match self.right {
                Some(ptr) => return (*ptr).find(target),
                None => return None
            }
        }
        None
    }

    pub unsafe fn insert(&mut self, new_node: *mut MemTreeNode) -> InsertResult {
        if self.content.addr < self.content.addr {
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
        (*new_node).parent = Some(self as *mut MemTreeNode);
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
                if Some(self as *mut MemTreeNode) == (*parent).left {
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
                (*right).left = Some(self as *mut MemTreeNode);
                self.parent = Some(right);
                (*right).parent = Some(self as *mut MemTreeNode);
                self.rotate_common(right);
            },
            None => ()
        }
    }

    pub unsafe fn rotate_right(&mut self) {
        match self.left {
            Some(left) => {
                self.left = (*left).right;
                (*left).right = Some(self as *mut MemTreeNode);
                self.parent = Some(left);
                (*left).parent = Some(self as *mut MemTreeNode);
                self.rotate_common(left);
            },
            None => ()
        }
    }

    pub fn parent(&self) -> Option<*mut MemTreeNode> {
        self.parent
    }

    unsafe fn rotate_common(&mut self, ptr: *mut MemTreeNode) {
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
                    if node == self as *mut MemTreeNode {
                        return Child::Left;
                    }
                }
                return Child::Right;
            }
            None => return Child::None
        }
    }

    unsafe fn grand_parent(&self) -> Option<*mut MemTreeNode> {
        match self.parent {
            Some(parent) => (*parent).parent,
            None => None
        }
    }

    unsafe fn sibling(&mut self) -> Option<*mut MemTreeNode> {
        match self.parent {
            Some(parent) => {
                if let Some(left) = (*parent).left {
                    if left == self as *mut MemTreeNode {
                        return (*parent).right;
                    }
                };
                if let Some(right) = (*parent).right {
                    if right == self as *mut MemTreeNode {
                        return (*parent).left;
                    }
                };
                None
            },
            None => None
        }
    }

    unsafe fn uncle(&mut self) -> Option<*mut MemTreeNode> {
        match self.parent {
            Some(parent) => (*parent).sibling(),
            None => None
        }
    }
}
