use std::cell::OnceCell;

use linter_adapter::context::{DriverContext, DriverContextWrapper};
use linter_api::context::AstContext;
use rustc_lint::LintStore;
use rustc_middle::ty::TyCtxt;

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

impl<'ast, 'tcx> DriverContext<'ast> for RustcContext<'ast, 'tcx> {
    fn emit_lint(&self, _lint: &'static linter_api::lint::Lint, _msg: &str, _span: &linter_api::ast::Span<'ast>) {
        todo!()
    }

    fn get_span(&'ast self, _owner: &linter_api::ast::SpanOwner) -> &'ast linter_api::ast::Span<'ast> {
        todo!()
    }

    fn span_snippet(&self, _span: &linter_api::ast::Span) -> Option<&'ast str> {
        todo!()
    }
}
