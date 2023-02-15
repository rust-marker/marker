use std::cell::OnceCell;

use marker_adapter::context::{DriverContext, DriverContextWrapper};
use marker_api::{
    ast::{
        item::{Body, ItemKind},
        BodyId, ExprId, ItemId, Span, SpanOwner, SymbolId,
    },
    context::AstContext,
    lint::Lint,
};
use rustc_lint::LintStore;
use rustc_middle::ty::TyCtxt;

use crate::conversion::{marker::MarkerConverter, rustc::RustcConverter};

use self::storage::Storage;

pub mod storage;

/// This is the central context for the rustc driver and the struct providing the
/// callback implementation for [`AstContext`](`marker_api::context::AstContext`).
///
/// The struct intentionally only stores the [`TyCtxt`] and [`LintStore`] and not
/// a [`LateContext`](`rustc_lint::LateContext`) as the late context operates on
/// the assumption that every AST node is only checked in the specific `check_`
/// function. This will in contrast convert the entire crate at once and might
/// also jump around inside the AST if a lint crate requests that. This also has
/// the added benefit that we can use the `'tcx` lifetime for them.
pub struct RustcContext<'ast, 'tcx> {
    pub rustc_cx: TyCtxt<'tcx>,
    pub lint_store: &'tcx LintStore,
    pub storage: &'ast Storage<'ast>,
    pub marker_converter: MarkerConverter<'ast, 'tcx>,
    pub rustc_converter: RustcConverter<'ast, 'tcx>,

    /// This is the [`AstContext`] wrapping callbacks to this instance of the
    /// [`RustcContext`]. The once cell will be set immediately after the creation
    /// which makes it safe to access afterwards.
    ast_cx: OnceCell<&'ast AstContext<'ast>>,
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    pub fn new(rustc_cx: TyCtxt<'tcx>, lint_store: &'tcx LintStore, storage: &'ast Storage<'ast>) -> &'ast Self {
        // Create context
        let driver_cx = storage.alloc(Self {
            rustc_cx,
            lint_store,
            storage,
            marker_converter: MarkerConverter::new(rustc_cx, storage),
            rustc_converter: RustcConverter::new(rustc_cx, storage),
            ast_cx: OnceCell::new(),
        });

        // Create and link `AstContext`
        let callbacks_wrapper = storage.alloc(DriverContextWrapper::new(driver_cx));
        let callbacks = storage.alloc(callbacks_wrapper.create_driver_callback());
        let ast_cx = storage.alloc(AstContext::new(callbacks));
        driver_cx.ast_cx.set(ast_cx).unwrap();

        driver_cx
    }

    pub fn ast_cx(&self) -> &'ast AstContext<'ast> {
        // The `OnceCell` is filled in the new function and can never be not set.
        self.ast_cx.get().unwrap()
    }
}

impl<'ast, 'tcx: 'ast> DriverContext<'ast> for RustcContext<'ast, 'tcx> {
    fn emit_lint(&'ast self, api_lint: &'static Lint, msg: &str, api_span: &Span<'ast>) {
        let rustc_lint = self.rustc_converter.to_lint(api_lint);
        self.rustc_cx.struct_span_lint_hir(
            rustc_lint,
            rustc_hir::CRATE_HIR_ID,
            self.rustc_converter.to_span(api_span),
            msg,
            |diag| diag,
        );
    }

    fn item(&'ast self, api_id: ItemId) -> Option<ItemKind<'ast>> {
        let rustc_id = self.rustc_converter.to_item_id(api_id);
        let rust_item = self.rustc_cx.hir().item(rustc_id);
        self.marker_converter.to_item(rust_item)
    }

    fn body(&'ast self, id: BodyId) -> &'ast Body<'ast> {
        let rustc_body = self.rustc_cx.hir().body(self.rustc_converter.to_body_id(id));
        self.marker_converter.to_body(rustc_body)
    }

    fn get_span(&'ast self, owner: &SpanOwner) -> &'ast Span<'ast> {
        let rustc_span = match owner {
            SpanOwner::Item(item) => self.rustc_cx.hir().item(self.rustc_converter.to_item_id(*item)).span,
            SpanOwner::SpecificSpan(span_id) => self.rustc_converter.to_span_from_id(*span_id),
        };
        self.storage.alloc(self.marker_converter.to_span(rustc_span))
    }

    fn span_snippet(&self, _span: &Span) -> Option<&'ast str> {
        todo!()
    }

    fn symbol_str(&'ast self, api_id: SymbolId) -> &'ast str {
        let sym = self.rustc_converter.to_symbol(api_id);
        // The lifetime is fake, as documented in [`rustc_span::Span::as_str()`].
        // It'll definitely live longer than the `'ast` lifetime, it's transmuted to.
        let rustc_str: &str = sym.as_str();
        // # Safety
        // `'ast` is shorter than `'tcx` or any rustc lifetime. This transmute
        // in combination with the comment above is therefore safe.
        let api_str: &'ast str = unsafe { std::mem::transmute(rustc_str) };
        api_str
    }

    fn resolve_method_target(&'ast self, _id: ExprId) -> ItemId {
        todo!()
    }
}
