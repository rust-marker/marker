//! This module is responsible for the [`MarkerContext`] struct and related plumbing.
//! Items in this module are generally unstable, with the exception of the
//! exposed interface of [`MarkerContext`].
//!
//! Checkout the documentation of `marker_adapter::context` for an explanation
//! of how the backend of these structs is implemented.

use std::{cell::RefCell, mem::transmute};

use crate::{
    common::{ExpnId, ExprId, ItemId, Level, MacroReport, SpanId, SymbolId, TyDefId},
    diagnostic::{Diagnostic, DiagnosticBuilder, EmissionNode},
    ffi,
    sem::{self, TyKind},
    span::{ExpnInfo, FileInfo, FilePos, Span, SpanPos, SpanSource},
    Lint,
};

mod map;
pub use map::*;

thread_local! {
    /// **Warning**
    ///
    /// These lifetimes are fake. This [`MarkerContext`] has the `'ast` lifetime in
    /// in both places. `'static` is required to store it in a static thread local
    /// value. The lifetimes are modified and guarded by [`set_ast_cx`] and
    /// [`with_cx`]
    ///
    /// See: `./docs/internal/driver-info.md` for more context
    #[doc(hidden)]
    static AST_CX: RefCell<Option<&'static MarkerContext<'static>>> = RefCell::new(None);
}

/// **Warning**
///
/// This function is unstable and only exported, to enable the adapter to set
/// the [`MarkerContext`] for a lint crate. Calling it from outside sources can
/// lead to undefined behavior.
///
/// See: `./docs/internal/driver-info.md` for more context
#[doc(hidden)]
pub fn set_ast_cx<'ast>(cx: &'ast MarkerContext<'ast>) {
    // Safety:
    // This `transmute` erases the `'ast` lifetime. This is uncool, but sadly
    // necessary to store the reference [`AST_CX`]. All accesses are guarded by
    // the [`with_cx`] function, which resets the lifetime to <= `'ast`.
    let cx_static: &'static MarkerContext<'static> = unsafe { transmute(cx) };
    AST_CX.with(|cx| cx.replace(Some(cx_static)));
}

/// This function provides the current [`MarkerContext`]. This function requires an
/// AST node as a source for its lifetime. In most cases, calling it is as simple
/// as this function:
///
/// ```ignore
/// pub fn span(&self) -> &Span<'ast> {
///     with_cx(self, |cx| cx.span(self.id))
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
    F: FnOnce(&'src MarkerContext<'ast>) -> R,
    'static: 'src,
{
    AST_CX.with(|cx| {
        let cx_static: &'static MarkerContext<'static> = cx
            .borrow()
            .expect("`with_cx` should only be called by nodes once the context has been set");
        // Safety:
        // This just recreates the lifetimes that were erased in [`set_ast_cx`].
        // See the referenced docs for a full explanation.
        let cx_ast: &'src MarkerContext<'ast> = unsafe { transmute(cx_static) };

        f(cx_ast)
    })
}

/// This context will be passed to each [`LintPass`](`super::LintPass`) call to enable the user
/// to emit lints and to retrieve nodes by the given ids.
#[repr(C)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct MarkerContext<'ast> {
    ast: AstMap<'ast>,
    callbacks: MarkerContextCallbacks<'ast>,
}

impl<'ast> std::fmt::Debug for MarkerContext<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarkerContext").finish()
    }
}

impl<'ast> MarkerContext<'ast> {
    /// Returns an [`AstMap`] instance, which can be used to retrieve AST nodes
    /// by their ids.
    pub fn ast(&self) -> &AstMap<'ast> {
        &self.ast
    }

    /// This function is used to emit a lint.
    ///
    /// Every lint emission, is bound to one specific node in the AST. This
    /// node is used to check the lint level and is the default [`Span`] of
    /// the diagnostic message. See [`EmissionNode`] for more information.
    /// The [`Span`] can be overwritten with [`DiagnosticBuilder::span`].
    ///
    /// The message parameter, will be the main message of the created diagnostic.
    /// This message and all messages emitted as part of the created diagnostic
    /// should start with a lower letter, according to [rustc's dev guide].
    ///
    /// The function will return a [`DiagnosticBuilder`] which can be used to decorate
    /// the diagnostic message, with notes and help messages. These customizations can
    /// be moved into a conditional closure, to improve performance under some circumstances.
    /// See [`DiagnosticBuilder::decorate`] for more information.
    ///
    /// The diagnostic message will be emitted when the builder instance is dropped.
    ///
    /// [rustc's dev guide]: <https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-output-style-guide>
    ///
    /// ## Example 1
    ///
    /// ```
    /// # use marker_api::prelude::*;
    /// # marker_api::declare_lint!{
    /// #     /// Dummy
    /// #     LINT,
    /// #     Warn,
    /// # }
    /// # fn value_provider<'ast>(cx: &MarkerContext<'ast>, node: ExprKind<'ast>) {
    ///     cx.emit_lint(LINT, node, "<lint message>");
    /// # }
    /// ```
    ///
    /// The code above will roughly generate the following error message:
    ///
    /// ```text
    ///  warning: <lint message>        <-- The message that is set by this function
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | node
    ///   | ^^^^
    ///   |
    /// ```
    ///
    /// ## Example 2
    ///
    /// ```
    /// # use marker_api::prelude::*;
    /// # marker_api::declare_lint!{
    /// #     /// Dummy
    /// #     LINT,
    /// #     Warn,
    /// # }
    /// # fn value_provider<'ast>(cx: &MarkerContext<'ast>, node: ExprKind<'ast>) {
    ///     cx.emit_lint(LINT, node, "<lint message>").help("<text>");
    /// # }
    /// ```
    ///
    /// The [`DiagnosticBuilder::help`] will add a help message like this:
    ///
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | node
    ///   | ^^^^
    ///   |
    ///   = help: <text>        <-- The added help message
    /// ```
    ///
    /// ## Example 3
    ///
    /// Adding a help message using [`DiagnosticBuilder::decorate`]:
    ///
    /// ```
    /// # use marker_api::prelude::*;
    /// # marker_api::declare_lint!{
    /// #     /// Dummy
    /// #     LINT,
    /// #     Warn,
    /// # }
    /// # fn value_provider<'ast>(cx: &MarkerContext<'ast>, node: ExprKind<'ast>) {
    ///     cx.emit_lint(LINT, node, "<lint message>").decorate(|diag| {
    ///         // This closure is only called, if the diagnostic will be emitted.
    ///         // Here you can create a beautiful help message.
    ///         diag.help("<text>");
    ///     });
    /// # }
    /// ```
    ///
    /// This will create the same help message as in example 2, but it will be faster
    /// if the lint is suppressed. The emitted message would look like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | node
    ///   | ^^^^
    ///   |
    ///   = help: <text>        <-- The added help message
    /// ```
    pub fn emit_lint(
        &self,
        lint: &'static Lint,
        node: impl EmissionNode<'ast>,
        msg: impl Into<String>,
    ) -> DiagnosticBuilder<'ast> {
        let id = node.node_id();
        let span = node.span();
        if matches!(lint.report_in_macro, MacroReport::No) && span.is_from_expansion() {
            return DiagnosticBuilder::dummy();
        }
        if self.ast().lint_level_at(lint, &node) == Level::Allow {
            return DiagnosticBuilder::dummy();
        }

        DiagnosticBuilder::new(lint, id, msg.into(), span.clone())
    }

    pub(crate) fn emit_diagnostic<'a>(&self, diag: &'a Diagnostic<'a, 'ast>) {
        self.callbacks.call_emit_diagnostic(diag);
    }

    /// This function tries to resolve the given path to the corresponding [`TyDefId`].
    ///
    /// The slice might be empty if the path could not be resolved. This could be
    /// due to an error in the path or because the linted crate doesn't have the
    /// required dependency. The function can also return multiple [`TyDefId`]s,
    /// if there are multiple crates with different versions in the dependency tree.
    ///
    /// The returned ids are unordered and, depending on the driver, can also
    /// change during different calls. The slice should not be stored across
    /// `check_*` calls.
    ///
    /// Here is a simple example, how the method could be used:
    /// ```ignore
    /// // Get the type of an expression and check that it's an ADT
    /// if let SemTyKind::Adt(ty) = expr.ty() {
    ///     // Check if the id belongs to the path
    ///     if cx.resolve_ty_ids("example::path::Item").contains(&ty.ty_id()) {
    ///         // ...
    ///     }
    /// }
    /// ```
    pub fn resolve_ty_ids(&self, path: &str) -> &[TyDefId] {
        (self.callbacks.resolve_ty_ids)(self.callbacks.data, path.into()).get()
    }
}

impl<'ast> MarkerContext<'ast> {
    pub(crate) fn expr_ty(&self, expr: ExprId) -> TyKind<'ast> {
        self.callbacks.call_expr_ty(expr)
    }
    pub(crate) fn ty_implements_trait(
        &self,
        ty: sem::TyKind<'ast>,
        trait_ref: &sem::FfiUserDefinedTraitRef<'_>,
    ) -> bool {
        (self.callbacks.ty_implements_trait)(self.callbacks.data, ty, trait_ref)
    }

    // FIXME: This function should probably be removed in favor of a better
    // system to deal with spans. See rust-marker/marker#175
    pub(crate) fn span_snipped(&self, span: &Span<'ast>) -> Option<&'ast str> {
        (self.callbacks.span_snippet)(self.callbacks.data, span)
            .get()
            .map(ffi::FfiStr::get)
    }

    pub(crate) fn span(&self, span_id: SpanId) -> &'ast Span<'ast> {
        self.callbacks.call_span(span_id)
    }

    pub(crate) fn span_source(&self, span: &Span<'_>) -> SpanSource<'ast> {
        (self.callbacks.span_source)(self.callbacks.data, span)
    }
    pub(crate) fn span_pos_to_file_loc(&self, file: &FileInfo<'ast>, pos: SpanPos) -> Option<FilePos<'ast>> {
        (self.callbacks.span_pos_to_file_loc)(self.callbacks.data, file, pos).into()
    }
    pub(crate) fn span_expn_info(&self, src_id: ExpnId) -> Option<&'ast ExpnInfo<'ast>> {
        (self.callbacks.span_expn_info)(self.callbacks.data, src_id).into()
    }

    pub(crate) fn symbol_str(&self, sym: SymbolId) -> &'ast str {
        self.callbacks.call_symbol_str(sym)
    }

    #[allow(unused)] // Will be used later(or removed)
    pub(crate) fn resolve_method_target(&self, expr: ExprId) -> ItemId {
        self.callbacks.resolve_method_target(expr)
    }
}

/// This struct holds function pointers to driver implementations of required
/// functions. These can roughly be split into two categories:
///
/// 1. **Public utility**: These functions will be exposed to lint-crates via an [`MarkerContext`]
///    instance. Therefore, the function signature of these has to be stable, or at least be stable
///    for [`MarkerContext`].
/// 2. **Internal utility**: These functions are intended for internal usage inside the API or the
///    `marker_adapter` crate. Some nodes might also have a reference to these callbacks to request
///    additional information if required. These are not part of the stable API and can therefore be
///    changed.
///
/// Any changes to this struct will most likely require changes to the
/// `DriverContextWrapper` implementation in the `marker_adapter` crate. That
/// type provides a simple wrapper to avoid driver unrelated boilerplate code.
#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct MarkerContextCallbacks<'ast> {
    /// The data that will be used as the first argument for the callback functions.
    /// The content of this data is defined by the driver (or by marker_adapter on behalf
    /// of the driver)
    pub data: &'ast MarkerContextData,

    // Lint emission and information
    pub emit_diag: for<'a> extern "C" fn(&'ast MarkerContextData, &'a Diagnostic<'a, 'ast>),

    // Public utility
    pub resolve_ty_ids: extern "C" fn(&'ast MarkerContextData, path: ffi::FfiStr<'_>) -> ffi::FfiSlice<'ast, TyDefId>,

    // Internal utility
    pub expr_ty: extern "C" fn(&'ast MarkerContextData, ExprId) -> TyKind<'ast>,
    pub ty_implements_trait:
        extern "C" fn(&'ast MarkerContextData, sem::TyKind<'ast>, &sem::FfiUserDefinedTraitRef<'_>) -> bool,
    pub span: extern "C" fn(&'ast MarkerContextData, SpanId) -> &'ast Span<'ast>,
    pub span_snippet: extern "C" fn(&'ast MarkerContextData, &Span<'ast>) -> ffi::FfiOption<ffi::FfiStr<'ast>>,
    pub span_source: extern "C" fn(&'ast MarkerContextData, &Span<'_>) -> SpanSource<'ast>,
    pub span_pos_to_file_loc:
        extern "C" fn(&'ast MarkerContextData, &FileInfo<'ast>, SpanPos) -> ffi::FfiOption<FilePos<'ast>>,
    pub span_expn_info: extern "C" fn(&'ast MarkerContextData, ExpnId) -> ffi::FfiOption<&'ast ExpnInfo<'ast>>,
    pub symbol_str: extern "C" fn(&'ast MarkerContextData, SymbolId) -> ffi::FfiStr<'ast>,
    pub resolve_method_target: extern "C" fn(&'ast MarkerContextData, ExprId) -> ItemId,
}

impl<'ast> MarkerContextCallbacks<'ast> {
    fn call_emit_diagnostic<'a>(&self, diag: &'a Diagnostic<'a, 'ast>) {
        (self.emit_diag)(self.data, diag);
    }

    fn call_expr_ty(&self, expr: ExprId) -> TyKind<'ast> {
        (self.expr_ty)(self.data, expr)
    }
    fn call_span(&self, span_id: SpanId) -> &'ast Span<'ast> {
        (self.span)(self.data, span_id)
    }
    fn call_symbol_str(&self, sym: SymbolId) -> &'ast str {
        (self.symbol_str)(self.data, sym).get()
    }
    pub fn resolve_method_target(&self, expr: ExprId) -> ItemId {
        (self.resolve_method_target)(self.data, expr)
    }
}

/// This type is used by [`MarkerContextCallbacks`] as the first argument to every
/// function. For more information, see the documentation of the `data` field
/// or from `marker_adapter::context`.
///
/// This type should never be constructed and is only meant as a pointer
/// casting target.
#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct MarkerContextData {
    /// `#[repr(C)]` requires a field, to make this a proper type. Using usize
    /// ensures that the structs has the same alignment requirement as a pointer.
    ///
    /// This was a nice catch from `clippy::cast_ptr_alignment`. This should have been
    /// fine anyways, but better safe than sorry.
    _data: usize,
}
