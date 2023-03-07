use std::{cell::RefCell, mem::transmute};

use crate::{
    ast::{
        item::{Body, ItemKind},
        BodyId, ExprId, ItemId, Span, SpanOwner, SymbolId,
    },
    diagnostic::{Diagnostic, DiagnosticBuilder, EmissionNode},
    ffi,
    lint::{Level, Lint},
};

thread_local! {
    /// **Warning**
    ///
    /// These lifetimes are fake. This [`AstContext`] has the `'ast` lifetime in
    /// in both places. `'static` is required to store it in a static thread local
    /// value. The lifetimes are modified and guarded by [`set_ast_cx`] and
    /// [`with_cx`]
    ///
    /// See: `./docs/internal/driver-info.md` for more context
    #[doc(hidden)]
    static AST_CX: RefCell<Option<&'static AstContext<'static>>> = RefCell::new(None);
}

/// **Warning**
///
/// This function is unstable and only exported, to enable the adapter to set
/// the [`AstContext`] for a lint crate. Calling it from outside sources can
/// lead to undefined behavior.
///
/// See: `./docs/internal/driver-info.md` for more context
#[doc(hidden)]
pub fn set_ast_cx<'ast>(cx: &'ast AstContext<'ast>) {
    // Safety:
    // This `transmute` erases the `'ast` lifetime. This is uncool, but sadly
    // necessary to store the reference [`AST_CX`]. All accesses are guarded by
    // the [`with_cx`] function, which resets the lifetime to <= `'ast`.
    let cx_static: &'static AstContext<'static> = unsafe { transmute(cx) };
    AST_CX.with(|cx| cx.replace(Some(cx_static)));
}

/// This function provides the current [`AstContext`]. This function requires an
/// AST node as a source for its lifetime. In most cases, calling it is as simple
/// as this function:
///
/// ```ignore
/// pub fn span(&self) -> &Span<'ast> {
///     with_cx(self, |cx| cx.get_span(self.id))
/// }
/// ```
///
/// The taken lifetime `'src` is different from `'ast` as it would otherwise require
/// the API and user to always specify that the node reference also has the `'ast`
/// lifetime. This might be a bit less descriptive, but makes the interaction way
/// easier.
///
/// See: `./docs/internal/driver-info.md` for more context
pub(crate) fn with_cx<'src, 'ast: 'src, T, F, R>(_lifetime_src: &'src T, f: F) -> R
where
    F: FnOnce(&'src AstContext<'ast>) -> R,
    'static: 'src,
{
    AST_CX.with(|cx| {
        let cx_static: &'static AstContext<'static> = cx
            .borrow()
            .expect("`with_cx` should only be called by nodes once the context has been set");
        // Safety:
        // This just recreates the lifetimes that were erased in [`set_ast_cx`].
        // See the referenced docs for a full explanation.
        let cx_ast: &'src AstContext<'ast> = unsafe { transmute(cx_static) };

        f(cx_ast)
    })
}

/// This context will be passed to each [`LintPass`](`super::LintPass`) call to enable the user
/// to emit lints and to retieve nodes by the given ids.
#[repr(C)]
pub struct AstContext<'ast> {
    driver: &'ast DriverCallbacks<'ast>,
}

impl<'ast> std::fmt::Debug for AstContext<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstContext").finish()
    }
}

impl<'ast> std::hash::Hash for AstContext<'ast> {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl<'ast> std::cmp::PartialEq for AstContext<'ast> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl<'ast> std::cmp::Eq for AstContext<'ast> {}

#[cfg(feature = "driver-api")]
impl<'ast> AstContext<'ast> {
    pub fn new(driver: &'ast DriverCallbacks<'ast>) -> Self {
        Self { driver }
    }
}

impl<'ast> AstContext<'ast> {
    pub fn lint_level_at(&self, lint: &'static Lint, node: impl Into<EmissionNode>) -> Level {
        self.driver.call_lint_level_at(lint, node.into())
    }

    #[allow(clippy::needless_pass_by_value)] // `&impl ToSting`
    pub fn emit_lint<F>(
        &self,
        lint: &'static Lint,
        node: impl Into<EmissionNode>,
        msg: impl ToString,
        span: &Span<'ast>,
        decorate: F,
    ) where
        F: FnOnce(&mut DiagnosticBuilder<'ast>),
    {
        let node = node.into();
        if self.lint_level_at(lint, node) != Level::Allow {
            let mut builder = DiagnosticBuilder::new(lint, node, msg.to_string(), span.clone());
            decorate(&mut builder);
            builder.emit(self);
        }
    }

    pub(crate) fn emit_diagnostic<'a>(&self, diag: &'a Diagnostic<'a, 'ast>) {
        self.driver.call_emit_diagnostic(diag);
    }

    /// This returns the [`ItemKind`] belonging to the given [`ItemId`]. It can
    /// return `None` in special cases depending on the used driver.
    ///
    /// #### Driver information
    /// * Rustc's driver will always return a valid item.
    pub fn item(&self, id: ItemId) -> Option<ItemKind<'ast>> {
        self.driver.call_item(id)
    }

    pub fn body(&self, id: BodyId) -> &Body<'ast> {
        self.driver.call_body(id)
    }
}

impl<'ast> AstContext<'ast> {
    pub(crate) fn span_snipped(&self, span: &Span) -> Option<String> {
        self.driver.call_span_snippet(span)
    }

    pub(crate) fn get_span<T: Into<SpanOwner>>(&self, span_owner: T) -> &'ast Span<'ast> {
        self.driver.call_get_span(&span_owner.into())
    }

    pub(crate) fn symbol_str(&self, sym: SymbolId) -> &'ast str {
        self.driver.call_symbol_str(sym)
    }

    pub(crate) fn resolve_method_target(&self, expr: ExprId) -> ItemId {
        self.driver.resolve_method_target(expr)
    }
}

/// This struct holds function pointers to driver implementations of required
/// functions. These can roughly be split into two categories:
///
/// 1. **Public utility**: These functions will be exposed to lint-crates via
///     an [`AstContext`] instance. Therefore, the function signature of these
///     has to be stable, or at least be stable for [`AstContext`].
/// 2. **Internal utility**: These functions are intended for internal usage
///     inside the API or the `marker_adapter` crate. Some nodes might also have
///     a reference to these callbacks to request additional information if
///     required. These are not part of the stable API and can therefore be changed.
///
/// Any changes to this struct will most likely require changes to the
/// `DriverContextWrapper` implementation in the `marker_adapter` crate. That
/// type provides a simple wrapper to avoid driver unrelated boilerplate code.
#[repr(C)]
#[doc(hidden)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct DriverCallbacks<'ast> {
    /// This is a pointer to the driver context, provided to each function as
    /// the first argument. This is an untyped pointer, since the driver is
    /// unknown to the api and adapter. The context has to be casted into the
    /// driver-specific type by the driver. A driver is always guaranteed to
    /// get its own context.
    pub driver_context: &'ast (),

    // Lint emission and information
    pub lint_level_at: extern "C" fn(&'ast (), &'static Lint, EmissionNode) -> Level,
    pub emit_diag: for<'a> extern "C" fn(&'ast (), &'a Diagnostic<'a, 'ast>),

    // Public utility
    pub item: extern "C" fn(&'ast (), id: ItemId) -> ffi::FfiOption<ItemKind<'ast>>,
    pub body: extern "C" fn(&'ast (), id: BodyId) -> &'ast Body<'ast>,

    // Internal utility
    pub get_span: extern "C" fn(&'ast (), &SpanOwner) -> &'ast Span<'ast>,
    pub span_snippet: extern "C" fn(&'ast (), &Span) -> ffi::FfiOption<ffi::FfiStr<'ast>>,
    pub symbol_str: extern "C" fn(&'ast (), SymbolId) -> ffi::FfiStr<'ast>,
    pub resolve_method_target: extern "C" fn(&'ast (), ExprId) -> ItemId,
}

impl<'ast> DriverCallbacks<'ast> {
    fn call_lint_level_at(&self, lint: &'static Lint, node: EmissionNode) -> Level {
        (self.lint_level_at)(self.driver_context, lint, node)
    }

    fn call_emit_diagnostic<'a>(&self, diag: &'a Diagnostic<'a, 'ast>) {
        (self.emit_diag)(self.driver_context, diag);
    }

    fn call_item(&self, id: ItemId) -> Option<ItemKind<'ast>> {
        (self.item)(self.driver_context, id).copy()
    }
    fn call_get_span(&self, span_owner: &SpanOwner) -> &'ast Span<'ast> {
        (self.get_span)(self.driver_context, span_owner)
    }
    fn call_span_snippet(&self, span: &Span) -> Option<String> {
        let result: Option<ffi::FfiStr> = (self.span_snippet)(self.driver_context, span).into();
        result.map(|x| x.to_string())
    }
    fn call_symbol_str(&self, sym: SymbolId) -> &'ast str {
        (self.symbol_str)(self.driver_context, sym).get()
    }
    pub fn call_body(&self, id: BodyId) -> &'ast Body<'ast> {
        (self.body)(self.driver_context, id)
    }
    pub fn resolve_method_target(&self, expr: ExprId) -> ItemId {
        (self.resolve_method_target)(self.driver_context, expr)
    }
}
