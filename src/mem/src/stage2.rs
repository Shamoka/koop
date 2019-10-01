use crate::frame;
use crate::memtree::MemTree;
use crate::stage1;
use crate::AllocError;
use crate::area::Area;

const ALLOC_AREA: Area = Area::new(0o177777_042_000_000_000_0000, 0o001_000_000_000_0000);

pub struct Allocator {
    internal: stage1::Allocator,
    memory_tree: MemTree,
}

impl Allocator {
    pub fn new(mut stage1: stage1::Allocator) -> Result<Allocator, AllocError> {
        let node_alloc_area = Area::new(ALLOC_AREA.base.addr, frame::FRAME_SIZE);
        if let Err(error) = stage1.map(&node_alloc_area) {
            return Err(error);
        }
        Ok(Allocator {
            internal: stage1,
            memory_tree: unsafe { MemTree::new(&node_alloc_area) }
        })
    }
}
