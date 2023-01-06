mod common;
pub use common::*;

use crate::ffi::FfiSlice;

use self::item::ItemKind;

pub mod generic;
pub mod item;
pub mod pat;
pub mod ty;

#[derive(Debug)]
pub struct Crate<'ast> {
    id: CrateId,
    items: FfiSlice<'ast, ItemKind<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> Crate<'ast> {
    pub fn new(id: CrateId, items: &'ast [ItemKind<'ast>]) -> Self {
        Self {
            id,
            items: items.into(),
        }
    }
}

impl<'ast> Crate<'ast> {
    /// This returns the ID of this crate object.
    pub fn id(&self) -> CrateId {
        self.id
    }

    /// This is a list of all items in the root file of the crate. Nested items
    /// will be represented in the form of items and sub-items
    pub fn items(&self) -> &[ItemKind<'ast>] {
        self.items.get()
    }
}
