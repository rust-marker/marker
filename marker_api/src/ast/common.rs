mod ast_path;
pub use ast_path::*;

use std::fmt::Debug;

use crate::common::ItemId;

use super::generic::SynGenericArgs;

#[repr(C)]
#[derive(Debug)]
pub struct TraitRef<'ast> {
    item_id: ItemId,
    generics: SynGenericArgs<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitRef<'ast> {
    pub fn new(item_id: ItemId, generics: SynGenericArgs<'ast>) -> Self {
        Self { item_id, generics }
    }
}

impl<'ast> TraitRef<'ast> {
    pub fn trait_id(&self) -> ItemId {
        self.item_id
    }

    pub fn generics(&self) -> &SynGenericArgs<'ast> {
        &self.generics
    }
}
