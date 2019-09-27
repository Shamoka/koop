use crate::addr::Addr;

pub enum PageType {
    Simple
}

pub struct Page {
    pub addr: Addr,
    pub page_type: PageType
}

impl Page {
    pub const fn new(addr: usize, page_type: PageType) -> Page {
        Page {
            addr: Addr::new(addr),
            page_type: page_type
        }
    }
}
