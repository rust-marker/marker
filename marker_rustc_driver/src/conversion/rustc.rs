mod common;
mod unstable;

use std::cell::RefCell;

use marker_api::lint::Lint;
use rustc_hash::FxHashMap;

use crate::context::storage::Storage;

pub struct RustcConverter<'ast, 'tcx> {
    rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
    storage: &'ast Storage<'ast>,
    lints: RefCell<FxHashMap<&'static Lint, &'static rustc_lint::Lint>>,
}

impl<'ast, 'tcx> RustcConverter<'ast, 'tcx> {
    pub fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        Self {
            rustc_cx,
            storage,
            lints: RefCell::default(),
        }
    }
}
