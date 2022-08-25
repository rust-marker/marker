use std::cell::OnceCell;

use linter_adapter::context::{DriverContext, DriverContextWrapper};
use linter_api::{
    ast::{Span, SpanOwner},
    context::AstContext,
    lint::Lint,
};
use rustc_lint::LintStore;
use rustc_middle::ty::TyCtxt;

use crate::conversion::{to_api_span, to_rustc_item_id, to_rustc_lint, to_rustc_span, to_rustc_span_from_id};

use self::storage::Storage;

pub mod storage;

/// This is the central context for the rustc driver and the struct providing the
/// callback implementation for [`AstContext`][`linter_api::context::AstContext`].
///
/// The struct intentionally only stores the [`TyCtxt`] and [`LintStore`] and not
/// a [`LateContext`][`rustc_lint::LateContext`] as the late context operates on
/// the assumption that every AST node is only checked in the specific `check_`
/// function. This will in contrast convert the entire crate at once and might
/// also jump around inside the AST if a lint crate requests that. This also has
/// the added benefit that we can use the `'tcx` lifetime for them.
pub struct RustcContext<'ast, 'tcx> {
    pub rustc_cx: TyCtxt<'tcx>,
    pub lint_store: &'tcx LintStore,
    pub storage: &'ast Storage<'ast>,
    /// This is the [`AstContext`] wrapping callbacks to this instance of the
    /// [`RustcContext`]. The once cell will be set imediatly after the creation
    /// which makes it safe to access afterwards. See
    ast_cx: OnceCell<&'ast AstContext<'ast>>,
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    pub fn new(rustc_cx: TyCtxt<'tcx>, lint_store: &'tcx LintStore, storage: &'ast Storage<'ast>) -> &'ast Self {
        // Create context
        let driver_cx = storage.alloc(|| Self {
            rustc_cx,
            lint_store,
            storage,
            ast_cx: OnceCell::new(),
        });

        // Create and link `AstContext`
        let callbacks_wrapper = storage.alloc(|| DriverContextWrapper::new(driver_cx));
        let callbacks = storage.alloc(|| callbacks_wrapper.create_driver_callback());
        let ast_cx = storage.alloc(|| AstContext::new(callbacks));
        driver_cx.ast_cx.set(ast_cx).unwrap();

        driver_cx
    }

    pub fn ast_cx(&self) -> &'ast AstContext<'ast> {
        // The `OnceCell` is filled in the new function and can never not be set.
        self.ast_cx.get().unwrap()
    }
}

impl<'ast, 'tcx: 'ast> DriverContext<'ast> for RustcContext<'ast, 'tcx> {
    fn emit_lint(&'ast self, api_lint: &'static Lint, msg: &str, api_span: &Span<'ast>) {
        let rustc_lint = to_rustc_lint(self, api_lint);
        self.rustc_cx.struct_span_lint_hir(
            rustc_lint,
            rustc_hir::CRATE_HIR_ID,
            to_rustc_span(self, api_span),
            |diag| {
                let mut diag = diag.build(msg);
                diag.emit();
            },
        );
    }

    fn get_span(&'ast self, owner: &SpanOwner) -> &'ast Span<'ast> {
        let rustc_span = match owner {
            SpanOwner::Item(item) => self.rustc_cx.hir().item(to_rustc_item_id(self, *item)).span,
            SpanOwner::SpecificSpan(span_id) => to_rustc_span_from_id(self, *span_id),
        };
        to_api_span(self, rustc_span)
    }

    fn span_snippet(&self, _span: &Span) -> Option<&'ast str> {
        todo!()
    }
}
