//! This module and its sub modules form the translation layer from rustc's
//! internal representation to markers representation. All conversion methods
//! are implemented as methods of the [`MarkerConversionContext`] to group them
//! together and share access to common objects easily.

mod common;
mod expr;
mod generics;
mod item;
mod pat;
mod stmts;
mod ty;

use std::cell::RefCell;

use crate::context::storage::Storage;
use marker_api::ast::{
    expr::ExprKind,
    item::{Body, ItemKind},
    BodyId, Crate, ExprId, ItemId, SymbolId,
};
use rustc_hash::FxHashMap;
use rustc_hir as hir;

pub struct MarkerConversionContext<'ast, 'tcx> {
    rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
    storage: &'ast Storage<'ast>,

    // Converted nodes cache
    items: RefCell<FxHashMap<ItemId, ItemKind<'ast>>>,
    bodies: RefCell<FxHashMap<BodyId, &'ast Body<'ast>>>,
    exprs: RefCell<FxHashMap<ExprId, ExprKind<'ast>>>,
    num_symbols: RefCell<FxHashMap<u32, SymbolId>>,

    // Context information
    /// This holds the [`hir::BodyId`] of the body that is currently being
    /// converted. This may be [`None`] for items, but should always be [`Some`]
    /// for expressions, since they can (AFAIK) only occur inside bodies.
    /// Individual expressions can be requested via the driver context, however,
    /// this driver only provides IDs of converted expressions, meaning that
    /// the requested expression would be returned from cache and not
    /// require additional translations.
    rustc_body: RefCell<Option<hir::BodyId>>,
    /// Requested on demand from rustc using a [`hir::BodyId`] see
    /// [`MarkerConversionContext::rustc_body`] for more information
    rustc_ty_check: RefCell<Option<&'tcx rustc_middle::ty::TypeckResults<'tcx>>>,
}

// General util functions
impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    pub fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        Self {
            rustc_cx,
            storage,
            items: RefCell::default(),
            bodies: RefCell::default(),
            exprs: RefCell::default(),
            num_symbols: RefCell::default(),
            rustc_body: RefCell::default(),
            rustc_ty_check: RefCell::default(),
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
