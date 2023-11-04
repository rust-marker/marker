//! A module containing the AST of Marker, which is the main syntactic
//! representation of the written code.

mod common;
mod expr;
mod generic;
mod item;
mod pat;
mod stmt;
mod ty;
pub use common::*;
pub use expr::*;
pub use generic::*;
pub use item::*;
pub use pat::*;
pub use stmt::*;
pub use ty::*;

use crate::common::CrateId;

#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct Crate<'ast> {
    id: CrateId,
    root_mod: ModItem<'ast>,
}

impl<'ast> Crate<'ast> {
    /// This returns the ID of this crate object.
    pub fn id(&self) -> CrateId {
        self.id
    }

    /// Returns the root module of the crate.
    pub fn root_mod(&self) -> &ModItem<'ast> {
        &self.root_mod
    }
}
