use crate::memtree::MemTreeNode;
use crate::pool::Pool;
use crate::stage1;
use crate::AllocError;
use crate::area::Area;
use crate::buddies::Buddies;

const ALLOC_AREA: Area = Area::new(0o042_000_000_000_0000, 0o010_0000);
const POOL_AREA: Area = Area::new(0o042_000_000_010_0000, 0o001_0000);

pub struct Allocator {
    internal: stage1::Allocator,
    buddies: Buddies,
}

impl Allocator {
    pub fn new(mut stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
        if let Err(error) = stage1.map(&ALLOC_AREA) {
            return Err(error);
        }
        if let Err(error) = stage1.map(&POOL_AREA) {
            return Err(error);
        }
        let mut allocator = Allocator {
            internal: stage1,
            buddies: unsafe { Buddies::new(Pool::<MemTreeNode>::new(&POOL_AREA, &ALLOC_AREA)) }
        };
        if let None = allocator.buddies.get_block(&ALLOC_AREA) {
            return Err(AllocError::OutOfMemory);
        }
        Ok(allocator)
    }
}
