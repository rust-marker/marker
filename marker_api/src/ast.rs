//! A module containing the AST of Marker, which is the main syntactic
//! representation of the written code.

mod common;
pub use common::*;

mod expr;
pub use expr::*;
mod generic;
pub use generic::*;
mod item;
pub use item::*;
mod pat;
pub use pat::*;
mod stmt;
pub use stmt::*;
mod ty;
pub use ty::*;

use crate::{common::CrateId, ffi::FfiSlice};

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
