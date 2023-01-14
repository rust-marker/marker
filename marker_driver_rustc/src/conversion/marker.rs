//! This module and its sub modules form the translation layer from rustc's
//! internal representation to markers representation. All conversion methods
//! are implemented as methods of the [`MarkerConversionContext`] to group them
//! together and share access to common objects easily.

mod common;
mod generics;
mod item;
mod pat;
mod ty;

use std::cell::RefCell;

use crate::context::storage::Storage;
use marker_api::ast::{item::ItemKind, Crate, ItemId, SymbolId};
use rustc_hash::FxHashMap;
use rustc_hir as hir;

pub struct MarkerConversionContext<'ast, 'tcx> {
    rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
    storage: &'ast Storage<'ast>,
    items: RefCell<FxHashMap<ItemId, ItemKind<'ast>>>,
    num_symbols: RefCell<FxHashMap<u32, SymbolId>>,
}

// General util functions
impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    pub fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        Self {
            rustc_cx,
            storage,
            items: RefCell::default(),
            num_symbols: RefCell::default(),
        }
    }

    #[must_use]
    fn alloc<F, T>(&self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.storage.alloc(f)
    }

    #[must_use]
    fn alloc_slice_iter<T, I>(&self, iter: I) -> &'ast [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.storage.alloc_slice_iter(iter)
    }
}

impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    #[must_use]
    pub fn to_crate(
        &self,
        rustc_crate_id: hir::def_id::CrateNum,
        rustc_root_mod: &'tcx hir::Mod<'tcx>,
    ) -> &'ast Crate<'ast> {
        self.alloc(|| Crate::new(self.to_crate_id(rustc_crate_id), self.to_items(rustc_root_mod.item_ids)))
    }
}
