mod common;
mod unstable;

use std::cell::RefCell;

use marker_api::lint::Lint;
use rustc_hash::FxHashMap;

use crate::context::storage::Storage;

thread_local! {
    /// This maps marker lints to lint instances used by rustc.
    ///
    /// Rustc requires lints to be registered, before the lint pass is run. This
    /// is a problem for this conversion setup, as the used `*Converter` structs
    /// require the `'ast` lifetime. Storing this specific map outside the struct
    /// and providing a static conversion method, is simply a hack to allow the
    /// early conversion of lints, so they can be registered.
    ///
    /// If we run into more problems like this, we might have to rethink the structure
    /// again... let's just hope this doesn't happen!
    static LINTS_MAP: RefCell<FxHashMap<&'static Lint, &'static rustc_lint::Lint>> = RefCell::default();
}

pub struct RustcConverter<'ast, 'tcx> {
    rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
    storage: &'ast Storage<'ast>,
}

impl<'ast, 'tcx> RustcConverter<'ast, 'tcx> {
    pub fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        Self { rustc_cx, storage }
    }
}
