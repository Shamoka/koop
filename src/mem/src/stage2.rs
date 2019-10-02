use crate::stage1;
use crate::AllocError;
use crate::buddies::Buddies;

pub struct Allocator {
    internal: stage1::Allocator,
    buddies: Buddies,
}

impl Allocator {
    pub fn new(mut stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
        let mut allocator = Allocator {
            internal: stage1,
            buddies: Buddies::new()
        };
        Ok(allocator)
    }
}
