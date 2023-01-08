//! This module and its sub modules form the translation layer from rustc's
//! internal representation to markers representation. All conversion methods
//! are implemented as methods of the [`RustcConversionContext`] to group them
//! together and share access to common objects easily.

use crate::context::storage::Storage;
use rustc_hir as hir;

pub struct RustcConversionContext<'ast, 'tcx> {
    pub rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
    pub storage: &'ast Storage<'ast>,
}

// General util functions
impl<'ast, 'tcx> RustcConversionContext<'ast, 'tcx> {
    pub fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        Self { rustc_cx, storage }
    }

    #[must_use]
    #[expect(dead_code, reason = "WIP")]
    fn alloc<F, T>(&self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.storage.alloc(f)
    }

    #[must_use]
    #[expect(dead_code, reason = "WIP")]
    fn alloc_slice_iter<T, I>(&'ast self, iter: I) -> &'ast [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.storage.alloc_slice_iter(iter)
    }
}

impl<'ast, 'tcx> RustcConversionContext<'ast, 'tcx> {
    pub fn conv_krate(&self, _rustc_crate_id: hir::def_id::CrateNum, _rustc_root_mod: &'tcx hir::Mod<'tcx>) {}
}
