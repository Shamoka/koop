use crate::block::Block;

use core::cmp::Ordering;

use core::mem::size_of;

pub enum TakeResult<'a> {
    Node(*mut Node<'a>),
    Block(Block),
    Empty
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Color {
    Red,
    Black,
}

#[derive(Copy, Clone)]
pub struct Tree<'a> {
    root: NodeType<'a>,
    pub block: Option<Block>,
}

#[derive(Debug, PartialEq)]
pub struct Node<'a> {
    pub content: Block,
    left: NodeType<'a>,
    right: NodeType<'a>,
    parent: NodeType<'a>,
    color: Color
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum NodeType<'a> {
    Leaf(*mut Node<'a>),
    Node(*mut Node<'a>),
    Nil
}

impl<'a> NodeType<'a> {
    unsafe fn is_node(&self) -> bool {
        match *self {
            NodeType::Node(_) => true,
            _ => false
        }
    }

    unsafe fn set_color(&mut self, new_color: Color) {
        match *self {
            NodeType::Node(node) => (*node).color = new_color,
            NodeType::Leaf(_) => {
                if let Color::Red = new_color {
                    panic!("Trying to turn a leaf node red");
                }
            }
            NodeType::Nil => panic!("Setting nil node to red")
        }
    }

    unsafe fn ptr(&self) -> *mut Node<'a> {
        match *self {
            NodeType::Node(ptr) => ptr,
            NodeType::Leaf(ptr) => ptr,
            NodeType::Nil => 0 as *mut Node
        }
    }

    pub unsafe fn get_color(&self) -> Color {
        match *self {
            NodeType::Node(node) => (*node).color,
            NodeType::Leaf(_) => Color::Black,
            NodeType::Nil => panic!("Color get nil")
        }
    }

    pub unsafe fn parent_ptr(&self) -> *mut Node<'a> {
        match *self {
            NodeType::Node(ptr) => (*ptr).parent.ptr(),
            NodeType::Leaf(ptr) => ptr,
            NodeType::Nil => 0 as *mut Node
        }
    }

    pub unsafe fn set_parent(&mut self, new_parent: &NodeType<'a>) {
        match *self {
            NodeType::Node(node) => (*node).parent = *new_parent,
            NodeType::Leaf(ref mut leaf) => *leaf = new_parent.ptr(),
            NodeType::Nil => panic!("Parent set nil")
        }
    }

    pub unsafe fn grand_parent(&self) -> Option<&'a mut NodeType<'a>> {
        let parent_node = self.parent_ptr();
        match parent_node.is_null() {
            true => None,
            false => {
                match (*parent_node).parent {
                    NodeType::Node(_) => Some(&mut (*parent_node).parent),
                    _ => None
                }
            }
        }
    }

    pub unsafe fn left(&self) -> &'a mut NodeType<'a> {
        match *self {
            NodeType::Node(ptr) => &mut (*ptr).left,
            _ => panic!("Accessing null ptr in memory tree: left")
        }
    }

    pub unsafe fn right(&self) -> &'a mut NodeType<'a> {
        match *self {
            NodeType::Node(ptr) => &mut (*ptr).right,
            _ => panic!("Accessing null ptr in memory tree: right")
        }
    }

    pub unsafe fn sibling(&self) -> Option<&'a mut NodeType<'a>> {
        let parent_node = self.parent_ptr();
        match parent_node.is_null() {
            true => None,
            false => {
                if (*parent_node).left == *self {
                    Some(&mut (*parent_node).right)
                } else {
                    Some(&mut (*parent_node).left)
                }
            }
        }
    }

    pub unsafe fn uncle(&self) -> Option<&'a mut NodeType<'a>> {
        match self.grand_parent() {
            Some(gp) => {
                if gp.left().ptr() == self.parent_ptr() {
                    Some(gp.right())
                } else {
                    Some(gp.left())
                }
            },
            None => None
        }
    }

    pub unsafe fn content(&self) -> &'a mut Block {
        match *self {
            NodeType::Node(ptr) => &mut (*ptr).content,
            _ => panic!("Accessing the content of a null node in memory tree")
        }
    }

    pub unsafe fn leftmost(&self) -> NodeType<'a> {
        if self.left().is_node() {
            return self.left().leftmost();
        }
        return *self;
    }

    pub unsafe fn bst_insert(&mut self, node: &mut NodeType<'a>) {
        if node.content().addr < self.content().addr {
            if self.left().is_node() {
                return self.left().bst_insert(node);
            } else {
                *self.left() = *node;
            }
        } else if node.content().addr > self.content().addr {
            if self.right().is_node() {
                return self.right().bst_insert(node);
            } else {
                *self.right() = *node;
            }
        }
        node.set_parent(self);
        *node.left() = NodeType::Leaf(node.ptr());
        *node.right() = NodeType::Leaf(node.ptr());
        node.set_color(Color::Red);
    }

    unsafe fn rotate_left(&mut self) {
        let mut new_node = *self.right();
        let parent = match self.parent_ptr().is_null() {
            true => NodeType::Nil,
            false =>  NodeType::Node(self.parent_ptr())
        };

        if !new_node.is_node() {
            panic!("Trying to rotate a leaf node");
        }

        *self.right() = *new_node.left();
        *new_node.left() = *self;
        self.set_parent(&new_node);
        if self.right().is_node() {
            self.right().set_parent(&self);
        }
        if parent.is_node() {
            if *self == *parent.left() {
                *parent.left() = new_node;
            } else if *self == *parent.right() {
                *parent.right() = new_node;
            }
        }
        new_node.set_parent(&parent);
    }

    unsafe fn rotate_right(&mut self) {
        let mut new_node = *self.left();
        let parent = match self.parent_ptr().is_null() {
            true => NodeType::Nil,
            false =>  NodeType::Node(self.parent_ptr())
        };

        if !new_node.is_node() {
            panic!("Trying to rotate a leaf node");
        }

        *self.left() = *new_node.right();
        *new_node.right() = *self;
        self.set_parent(&new_node);
        if self.left().is_node() {
            self.left().set_parent(&self);
        }
        if parent.is_node() {
            if *self == *parent.left() {
                *parent.left() = new_node;
            } else if *self == *parent.right() {
                *parent.right() = new_node;
            }
        }
        new_node.set_parent(&parent);
    }

    pub unsafe fn repair(&mut self) {
        if self.parent_ptr().is_null() {
            return self.set_color(Color::Black);
        }
        if (*self.parent_ptr()).color == Color::Black {
            return ;
        }
        if let Some(uncle) = self.uncle() {
            if uncle.get_color() == Color::Red {
                return self.repair_3();
            }
        }
        self.repair_4();
    }

    unsafe fn repair_3(&mut self) {
        let parent_ptr = self.parent_ptr();
        if !parent_ptr.is_null() {
            (*parent_ptr).color = Color::Black;
            if let Some(uncle) = self.uncle() {
                uncle.set_color(Color::Black);
            }
            if let Some(grand_parent) = self.grand_parent() {
                grand_parent.set_color(Color::Red);
                grand_parent.repair();
            }
        }
    }

    unsafe fn repair_4(&mut self) {
        let parent_ptr = self.parent_ptr();

        if let Some(&mut grand_parent) = self.grand_parent() {
            let mut parent = NodeType::Node(parent_ptr);
            if *self == *parent.right() && parent == *grand_parent.left() {
                parent.rotate_left();
                self.left().repair_4_2();
            } else if *self == *parent.left() && parent == *grand_parent.right() {
                parent.rotate_right();
                self.right().repair_4_2();
            } else {
                self.repair_4_2();
            }
        }
    }

    unsafe fn repair_4_2(&mut self) {
        if let Some(&mut mut grand_parent) = self.grand_parent() {
            let mut parent = NodeType::Node(self.parent_ptr());
            if *self == *parent.left() {
                grand_parent.rotate_right();
            } else {
                grand_parent.rotate_left();
            }
            parent.set_color(Color::Black);
            grand_parent.set_color(Color::Red);
        }
    }

    pub unsafe fn delete(&mut self, key: usize) -> Option<(*mut Node<'a>, *mut Node<'a>)> {
        match self.is_node() {
            true => {
                match self.content().addr.cmp(&key) {
                    Ordering::Less => self.right().delete(key),
                    Ordering::Greater => self.left().delete(key),
                    Ordering::Equal => {
                        if self.left().is_node() && self.right().is_node() {
                            let mut leftmost = self.right().leftmost();
                            *self.content() = *leftmost.content();
                            leftmost.delete(self.content().addr)
                        } else  {
                            let child = match self.left().is_node() {
                                true => self.left(),
                                false => self.right()
                            };
                            self.replace_child(child);
                            if self.get_color() == Color::Black {
                                if child.get_color() == Color::Red {
                                    child.set_color(Color::Black);
                                } else {
                                    child.delete_1()
                                }
                            }
                            match *self {
                                NodeType::Node(ptr) => Some((ptr, child.ptr())),
                                NodeType::Leaf(_) => Some((self.ptr(), child.ptr())),
                                _ => None
                            }
                        }
                    }
                }
            },
            false => None
        }
    }

    unsafe fn replace_child(&mut self, child: &mut NodeType<'a>) {
        let parent = match self.parent_ptr().is_null() {
            true => NodeType::Nil,
            false => NodeType::Node(self.parent_ptr())
        };
        child.set_parent(&parent);
        if parent.is_node() {
            if *parent.left() == *self {
                if child.is_node() {
                    *parent.left() = NodeType::Node(child.ptr());
                } else {
                    *parent.left() = NodeType::Leaf(parent.ptr());
                }
            } else {
                if child.is_node() {
                    *parent.right() = NodeType::Node(child.ptr());
                } else {
                    *parent.right() = NodeType::Leaf(parent.ptr());
                }
            }
        }
    }

    unsafe fn delete_1(&mut self) {
        if !self.parent_ptr().is_null() {
            self.delete_2();
        }
    }

    unsafe fn delete_2(&mut self) {
        if let Some(&mut mut s) = self.sibling() {
            let mut parent = NodeType::Node(self.parent_ptr());
            if s.get_color() == Color::Red {
                parent.set_color(Color::Red);
                s.set_color(Color::Black);
                if *self == *parent.left() {
                    parent.rotate_left();
                } else {
                    parent.rotate_right();
                }
            }
        }
        self.delete_3();
    }

    unsafe fn delete_3(&mut self) {
        if let Some(&mut mut s) = self.sibling() {
            if s.is_node() {
                let mut parent = NodeType::Node(self.parent_ptr());
                if parent.get_color() == Color::Black && s.get_color() == Color::Black
                    && s.right().get_color() == Color::Black && s.left().get_color() == Color::Black {
                        s.set_color(Color::Red);
                        parent.delete_1();
                    } else {
                        self.delete_4();
                }
            }
        }
    }

    unsafe fn delete_4(&mut self) {
        if let Some(&mut mut s) = self.sibling() {
            let mut parent = NodeType::Node(self.parent_ptr());
            if parent.get_color() == Color::Red && s.get_color() == Color::Black
                && s.right().get_color() == Color::Black && s.left().get_color() == Color::Black {
                    s.set_color(Color::Red);
                    parent.set_color(Color::Black);
                } else {
                    self.delete_5();
            }
        }
    }

    unsafe fn delete_5(&mut self) {
        if let Some(&mut mut s) = self.sibling() {
            if s.get_color() == Color::Black {
                let parent = NodeType::Node(self.parent_ptr());
                if *self == *parent.left() && s.right().get_color() == Color::Black
                    && s.left().get_color() == Color::Red {
                        s.set_color(Color::Red);
                        s.left().set_color(Color::Black);
                        s.rotate_right();
                    } else if *self == *parent.right() && s.left().get_color() == Color::Black
                        && s.right().get_color() == Color::Red {
                            s.set_color(Color::Red);
                            s.right().set_color(Color::Red);
                            s.rotate_left();
                    }
            }
        }
        self.delete_6();
    }

    unsafe fn delete_6(&mut self) {
        if let Some(&mut mut s) = self.sibling() {
            let mut parent = NodeType::Node(self.parent_ptr());
            s.set_color(parent.get_color());
            parent.set_color(Color::Black);
            if *self == *parent.left() {
                s.right().set_color(Color::Black);
                parent.rotate_left();
            } else {
                s.left().set_color(Color::Black);
                parent.rotate_right();
            }
        }
    }

    pub unsafe fn inspect(&self, depth: usize) {
        if self.is_node() {
            self.left().inspect(depth + 1);
            vga::println!("{}: {:?} {:?}", depth, self.get_color(), self.content());
            self.right().inspect(depth + 1);
        }
    }
}

impl<'a, 'b> Tree<'a> {
    pub const fn new() -> Tree<'a> {
        Tree {
            root: NodeType::Nil,
            block: None
        }
    }

    pub fn take(&mut self) -> TakeResult {
        match self.block.take() {
            Some(block) => TakeResult::Block(block),
            None => TakeResult::Empty
        }
    }

    pub fn insert_block(&mut self, new_block: &Block) -> bool {
        match self.block {
            Some(_) => return false,
            None => {
                self.block = Some(*new_block);
                return true;
            }
        }
    }

    pub fn delete(&mut self, key: usize) -> Option<*mut Node<'a>> {
        unsafe {
            match self.root.delete(key) {
                Some((ret, new_root_node)) => {
                    match new_root_node.is_null() {
                        true => self.root = NodeType::Nil,
                        false => {
                            let mut new_root = NodeType::Node(new_root_node);
                            while !new_root.parent_ptr().is_null() {
                                new_root = NodeType::Node(new_root.parent_ptr());
                            }
                            self.root = new_root;
                        }
                    };
                    Some(ret)
                },
                None => None
            }
        }
    }

    pub fn inspect(&self) {
        unsafe {
            if !self.root.is_node() {
                vga::println!("Empty!");
            } else {
                self.root.inspect(0);
            }
        }
    }

    pub fn insert(&mut self, node: *mut Node<'a>) {
        unsafe {
            let mut new_node = NodeType::Node(node);
            if self.root.is_node() {
                self.root.bst_insert(&mut new_node);
            } else {
                new_node.set_parent(&NodeType::Nil);
                *new_node.left() =  NodeType::Leaf(node);
                *new_node.right() = NodeType::Leaf(node);
                new_node.set_color(Color::Black);
                self.root = new_node;
            }
            new_node.repair();
            let mut new_root = NodeType::Node(new_node.ptr());
            while !new_root.parent_ptr().is_null() {
                new_root = NodeType::Node(new_root.parent_ptr());
            }
            self.root = new_root;
        }
    }
}
