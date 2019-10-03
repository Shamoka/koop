pub struct Stack<T: Sized + Copy + Clone> {
    root: Option<*mut StackNode<T>>,
    pool: Option<*mut StackNode<T>>,
    pool_count: usize
}

pub struct StackNode<T: Sized + Copy + Clone> {
    pub content: T,
    pub next: Option<*mut StackNode<T>>
}

impl<T: Sized + Copy + Clone> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            root: None,
            pool: None,
            pool_count: 0
        }
    }

    pub fn pool_count(&self) -> usize {
        self.pool_count
    }

    pub fn push(&mut self, value: T) -> bool {
        unsafe {
            if let Some(node) = self.unpool() {
                (*node).content = value;
                (*node).next = self.root;
                self.root = Some(node);
                true
            } else {
                false
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            match self.root.take() {
                Some(root) => {
                    self.root = (*root).next.take();
                    let content = (*root).content;
                    self.pool(root);
                    Some(content)
                },
                None => None
            }
        }
    }

    unsafe fn pool(&mut self, node: *mut StackNode<T>) {
        (*node).next = self.pool;
        self.pool = Some(node);
        self.pool_count += 1;
    }

    pub fn unpool(&mut self) -> Option<*mut StackNode<T>> {
        if self.pool_count == 0 {
            return None;
        }
        unsafe {
            match self.pool.take() {
                Some(node) => {
                    self.pool = (*node).next.take();
                    self.pool_count -= 1;
                    Some(node)
                },
                None => None
            }
        }
    }
}
