mod common;
pub use common::*;

use self::item::ItemType;

pub mod item;
pub mod ty;

#[derive(Debug)]
pub struct Crate<'ast> {
    id: CrateId,
    items: &'ast [ItemType<'ast>],
}

#[cfg(feature = "driver-api")]
impl<'ast> Crate<'ast> {
    pub fn new(id: CrateId, items: &'ast [ItemType<'ast>]) -> Self {
        Self { id, items }
    }
}

impl<'ast> Crate<'ast> {
    /// This returns the ID of this crate object.
    pub fn get_id(&self) -> CrateId {
        self.id
    }

    /// This is a list of all items in the root file of the crate. Nested items
    /// will be represented in the form of items and sub-items
    pub fn get_items(&self) -> &[ItemType<'ast>] {
        self.items
    }
}
