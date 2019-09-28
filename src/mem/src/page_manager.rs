use crate::area::Area;
use crate::AllocError;

#[derive(Debug)]
pub enum Error {
    Allocation(AllocError),
    InvalidPage,
    InvalidCall
}

pub trait Manager {
    type Backend;

    fn create() -> Result<Self::Backend, Error>;
    fn insert(&mut self, area: &Area) -> Result<(), Error>;
    fn remove(&mut self, area: &Area) -> Result<(), Error>;
    fn contains(&self, area: &Area) -> bool;
}
