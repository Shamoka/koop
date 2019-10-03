use crate::block::Block;

use core::ops::DerefMut;

#[derive(Copy, Clone)]
pub struct MemTree {
    root: *mut MemTreeNode,
    block: Option<Block>
}

pub enum TakeResult {
    Node(*mut MemTreeNode),
    Block(Block),
    Empty
}

#[derive(Copy, Clone, PartialEq)]
enum Color {
    Red,
    Black
}

#[derive(Copy, Clone)]
pub struct MemTreeNode {
    pub content: Block,
    pub left: *mut MemTreeNode,
    pub right: *mut MemTreeNode,
    pub parent: *mut MemTreeNode,
    color: Color
}

impl MemTree {
    pub const fn new() -> MemTree {
        MemTree {
            root: 0 as *mut MemTreeNode,
            block: None
        }
    }

    pub fn inspect(&mut self) {
        unsafe {
            if !self.root.is_null() {
                (*self.root).inspect()
            }
        }
    }

    pub fn take(&mut self) -> TakeResult {
        unsafe {
            match self.block.take() {
                Some(block) => TakeResult::Block(block),
                None => {
                    match self.root.is_null() {
                        false => TakeResult::Node((*self.root).remove()),
                        true => TakeResult::Empty
                    }
                }
            }
        }
    }

    pub fn insert_block(&mut self, block: &Block) -> bool {
        match self.block {
            Some(_) => false,
            None => {
                self.block = Some(*block);
                true
            }
        }
    }

    pub fn insert(&mut self, new_node: *mut MemTreeNode) {
        unsafe {
            match self.root.is_null() {
                false => {
                    (*self.root).insert(new_node);
                    (*new_node).repair();
                    let mut new_root = new_node;
                    while (*new_root).parent.is_null() == false {
                        new_root = (*new_root).parent;
                    }
                    self.root = new_root;
                },
                true => self.root = new_node
            }
        }
    }

    pub fn find(&mut self, addr: usize) -> Option<*mut MemTreeNode> {
        unsafe {
            match self.root.is_null() {
                false => (*self.root).find(addr),
                true => None
            }
        }
    }
}

impl MemTreeNode {
    pub fn init(&mut self, block: &Block) {
        self.content = *block;
        self.left = 0 as *mut MemTreeNode;
        self.right = 0 as *mut MemTreeNode;
        self.parent = 0 as *mut MemTreeNode;
        self.color = Color::Red;
    }

    pub unsafe fn inspect(&mut self) {
        if !self.left.is_null() {
            (*self.left).inspect();
        }
        vga::println!("{} {}", self.content.addr, self.content.size());
        if !self.right.is_null() {
            (*self.right).inspect();
        }
    }

    pub unsafe fn remove(&mut self) -> *mut MemTreeNode {
        if !self.left.is_null() && !self.right.is_null() {
            let leftmost = (*self.right).find_leftmost_child();
            self.content = (*leftmost).content;
            return (*leftmost).remove();
        }
        let child: *mut MemTreeNode;
        if !self.left.is_null() {
            child = self.left;
        } else {
            child = self.right;
        }
        self.replace_node(child);
        if self.color == Color::Black {
            if (*child).color == Color::Red {
                (*child).color = Color::Black;
            } else {
                (*child).delete_case_1();
            }
        }
        self as *mut MemTreeNode
    }


    pub unsafe fn find(&mut self, addr: usize) -> Option<*mut MemTreeNode> {
        if addr == self.content.addr {
            return Some(self as *mut MemTreeNode);
        }
        if addr < self.content.addr {
            if !self.left.is_null() {
                return (*self.left).find(addr);
            }
        } else if addr > self.content.addr {
            if !self.right.is_null() {
                return (*self.right).find(addr);
            }
        }
        None
    }

    pub unsafe fn insert(&mut self, new_node: *mut MemTreeNode) {
        if self.content.addr > (*new_node).content.addr {
            match self.left.is_null() {
                false => return (*self.left).insert(new_node),
                true => self.left = new_node
            };
        } else {
            match self.right.is_null() {
                false => return (*self.right).insert(new_node),
                true => self.right = new_node
            };
        }
        (*new_node).parent = self as *mut MemTreeNode;
    }

    pub unsafe fn repair(&mut self) {
        let uncle = self.uncle();
        match self.parent.is_null() {
            false => {
                if (*self.parent).color == Color::Black {
                    self.insert_case_2();
                }
                else if uncle.is_null() == false && (*uncle).color == Color::Red {
                    self.insert_case_3();
                }
                else {
                    self.insert_case_4();
                }
            },
            true => self.insert_case_1()
        }
    }

    unsafe fn replace_node(&mut self, child: *mut MemTreeNode) {
        (*child).parent = self.parent;
        if !self.parent.is_null() {
            if self as *mut MemTreeNode == (*self.parent).left {
                (*self.parent).left = child;
            } else if self as *mut MemTreeNode == (*self.parent).right {
                (*self.parent).right = child;
            }
        }
    }

    unsafe fn delete_case_1(&mut self) {
        if !self.parent.is_null() {
            self.delete_case_2();
        }
    }

    unsafe fn delete_case_2(&mut self) {
        let s = self.sibling();
        let parent = self.parent;
        let mut right = 0 as *mut MemTreeNode;
        let mut left = 0 as *mut MemTreeNode;
        if !parent.is_null() {
            right = (*parent).right;
            left = (*parent).left;
        }

        if !s.is_null() && (*s).color == Color::Red  {
            (*s).color = Color::Black;
            (*parent).color = Color::Red;
            if left == self as *mut MemTreeNode {
                (*parent).rotate_left();
            } else if right == self as *mut MemTreeNode {
                (*parent).rotate_right();
            }
        }
        self.delete_case_3();
    }

    unsafe fn delete_case_3(&mut self) {
        let s = self.sibling();
        let parent = self.parent;
        let mut right_color = Color::Black;
        let mut left_color = Color::Black;

        if !s.is_null() {
            if (*s).right.is_null() == false {
                right_color = (*(*s).right).color;
            }
            if !(*s).left.is_null() {
                left_color = (*(*s).left).color;
            }
        }

        if !parent.is_null() && !s.is_null() {
            if (*parent).color == Color::Black && (*s).color == Color::Black
                && left_color == Color::Black && right_color == Color::Black {
                    (*s).color = Color::Red;
                    (*parent).delete_case_1();
                } else {
                    self.delete_case_4();
            }
        }
    }

    unsafe fn delete_case_4(&mut self) {
        let s = self.sibling();
        let parent = self.parent;
        let right_color = (*(*s).right).color;
        let left_color = (*(*s).left).color;

        if (*parent).color == Color::Red && (*s).color == Color::Black
            && left_color == Color::Black && right_color == Color::Black {
                (*s).color = Color::Red;
                (*parent).color = Color::Black;
            } else {
                self.delete_case_5();
        }
    }

    unsafe fn delete_case_5(&mut self) {
        let s = self.sibling();
        let parent = self.parent;
        let sright = (*s).right;
        let sleft = (*s).left;
        let pright = (*parent).right;
        let pleft = (*parent).left;

        if (*s).color == Color::Black {
            if pleft == self as *mut MemTreeNode && (*sright).color == Color::Black
                && (*sleft).color == Color::Red {
                    (*s).color = Color::Red;
                    (*sleft).color = Color::Black;
                    (*s).rotate_right();
                } else if pright == self as *mut MemTreeNode && (*sleft).color == Color::Black
                    && (*sright).color == Color::Red  {
                        (*s).color = Color::Red;
                        (*sright).color = Color::Black;
                        (*s).rotate_left();
                }
        }
        self.delete_case_6();
    }

    unsafe fn delete_case_6(&mut self) {
        let s = self.sibling();
        let parent = self.parent;
        let sright = (*s).right;
        let sleft = (*s).left;
        let pleft = (*parent).left;

        (*s).color = (*parent).color;
        (*parent).color = Color::Black;
        if pleft == self as *mut MemTreeNode {
            (*sright).color = Color::Black;
            (*parent).rotate_left();
        } else {
            (*sleft).color = Color::Black;
            (*parent).rotate_right();
        }
    }

    unsafe fn insert_case_1(&mut self) {
        if self.parent.is_null() {
            self.color = Color::Black;
        }
    }

    unsafe fn insert_case_2(&mut self) {
        return;
    }

    unsafe fn insert_case_3(&mut self) {
        let parent = self.parent;
        let uncle = self.uncle();
        let grand_parent = self.grand_parent();

        (*parent).color = Color::Black;
        (*uncle).color = Color::Black;
        (*grand_parent).color = Color::Red;
        (*grand_parent).repair();
    }

    unsafe fn insert_case_4(&mut self) {
        let parent = self.parent;
        let mut n = self as *mut MemTreeNode;

        let mut grand_parent = self.grand_parent();
        if !grand_parent.is_null() {
            if (*parent).right == self as *mut MemTreeNode && parent == (*grand_parent).left {
                (*parent).rotate_left();
                n = self.left;
            } else if (*parent).left == self as *mut MemTreeNode && parent == (*grand_parent).right {
                (*parent).rotate_right();
                n = self.right;
            }
        }
        if !n.is_null() {
            (*n).insert_case_4_step_2();
        }
    }

    unsafe fn insert_case_4_step_2(&mut self) {
        let parent = self.parent;
        
        if !parent.is_null() {
            let grand_parent = (*parent).parent;
            if !grand_parent.is_null() {
                if (*parent).left == self as *mut MemTreeNode {
                    (*grand_parent).rotate_right();
                } else {
                    (*grand_parent).rotate_left();
                }
                (*grand_parent).color = Color::Red;
            }
            (*parent).color = Color::Black;
        }
    }

    unsafe fn find_leftmost_child(&mut self) -> *mut MemTreeNode {
        if self.left.is_null() {
            self as *mut MemTreeNode
        } else {
            (*self.left).find_leftmost_child()
        }
    }

    unsafe fn rotate_left(&mut self) {
        let new_node = self.right;
        let parent = self.parent;

        self.right = (*new_node).left;
        (*new_node).left = self as *mut MemTreeNode;
        self.parent = new_node;
        if !self.right.is_null() {
            (*self.right).parent = self as *mut MemTreeNode;
        }
        if !parent.is_null() {
            self.rotate_common(parent, new_node);
        }
        (*new_node).parent = parent;
    }

    unsafe fn rotate_right(&mut self) {
        let new_node = self.left;
        let parent = self.parent;

        self.left = (*new_node).right;
        (*new_node).right = self as *mut MemTreeNode;
        self.parent = new_node;
        if !self.left.is_null() {
            (*self.left).parent = self as *mut MemTreeNode;
        }
        if !parent.is_null() {
            self.rotate_common(parent, new_node);
        }
        (*new_node).parent = parent;
    }

    unsafe fn rotate_common(&mut self, parent: *mut MemTreeNode, new_node: *mut MemTreeNode) {
        let pleft = (*parent).left;
        let pright = (*parent).right;
        if pleft == self as *mut MemTreeNode {
            (*parent).left = new_node;
        } else if pright == self as *mut MemTreeNode {
            (*parent).right = new_node;
        }
    }

    unsafe fn grand_parent(&self) -> *mut MemTreeNode {
        if !self.parent.is_null() {
            (*self.parent).parent
        } else {
            0 as *mut MemTreeNode
        }
    }

    unsafe fn sibling(&mut self) -> *mut MemTreeNode {
        match self.parent.is_null() {
            false => {
                if (*self.parent).left == self as *mut MemTreeNode {
                    return (*self.parent).right;
                }
                else {
                    return (*self.parent).left;
                }
            },
            true => 0 as *mut MemTreeNode
        }
    }

    unsafe fn uncle(&mut self) -> *mut MemTreeNode {
        match self.parent.is_null() {
            false => (*self.parent).sibling(),
            true => 0 as *mut MemTreeNode
        }
    }
}
