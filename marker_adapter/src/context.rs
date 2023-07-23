#![allow(
    clippy::needless_lifetimes,
    reason = "the lifetimes are destroyed by unsafe, but help with readability"
)]

use marker_api::{
    ast::{
        item::{Body, ItemKind},
        ty::SemTyKind,
        BodyId, ExprId, ItemId, Span, SpanId, SymbolId, TyDefId,
    },
    context::DriverCallbacks,
    diagnostic::{Diagnostic, EmissionNode},
    ffi::{self, FfiOption},
    lint::{Level, Lint},
};

/// ### Safety
///
/// `&dyn` objects are theoretically not FFI safe since their type layout can
/// change and calling functions on them would require a stable ABI which Rust
/// doesn't provide.
///
/// In this case, the [`DriverContextWrapper`] will be passed as a `*const ()`
/// pointer to [`DriverCallbacks`] which will do nothing with this data other
/// than giving it back to functions declared in this module. Since the `&dyn`
/// object is created, only used here and everything is compiled during the same
/// compiler run, it should be safe to use `&dyn`.
#[repr(C)]
pub struct DriverContextWrapper<'ast> {
    driver_cx: &'ast dyn DriverContext<'ast>,
}

impl<'ast> DriverContextWrapper<'ast> {
    #[must_use]
    pub fn new(driver_cx: &'ast dyn DriverContext<'ast>) -> Self {
        Self { driver_cx }
    }

    #[must_use]
    pub fn create_driver_callback(&'ast self) -> DriverCallbacks<'ast> {
        DriverCallbacks {
            driver_context: unsafe { &*(self as *const DriverContextWrapper).cast::<()>() },
            lint_level_at,
            emit_diag,
            item,
            body,
            resolve_ty_ids,
            expr_ty,
            span,
            span_snippet,
            symbol_str,
            resolve_method_target,
        }
    }
}

#[allow(improper_ctypes_definitions, reason = "fp because `EmissionNode` are non-exhaustive")]
extern "C" fn lint_level_at(data: &(), lint: &'static Lint, node: EmissionNode) -> Level {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.lint_level_at(lint, node)
}

extern "C" fn emit_diag<'a, 'ast>(data: &(), diag: &Diagnostic<'a, 'ast>) {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.emit_diag(diag);
}

#[allow(improper_ctypes_definitions, reason = "fp because `ItemKind` is non-exhaustive")]
extern "C" fn item<'ast>(data: &(), id: ItemId) -> FfiOption<ItemKind<'ast>> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.item(id).into()
}

extern "C" fn body<'ast>(data: &(), id: BodyId) -> &'ast Body<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.body(id)
}

extern "C" fn resolve_ty_ids<'ast>(data: &(), path: ffi::FfiStr<'_>) -> ffi::FfiSlice<'ast, TyDefId> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.resolve_ty_ids((&path).into()).into()
}

#[allow(improper_ctypes_definitions, reason = "fp because `TyKind` is non-exhaustive")]
extern "C" fn expr_ty<'ast>(data: &(), expr: ExprId) -> SemTyKind<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.expr_ty(expr)
}

extern "C" fn span<'ast>(data: &(), span_id: SpanId) -> &'ast Span<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.span(span_id)
}

extern "C" fn span_snippet<'ast>(data: &(), span: &Span<'ast>) -> ffi::FfiOption<ffi::FfiStr<'ast>> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.span_snippet(span).map(Into::into).into()
}

extern "C" fn symbol_str<'ast>(data: &(), sym: SymbolId) -> ffi::FfiStr<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.symbol_str(sym).into()
}

extern "C" fn resolve_method_target(data: &(), id: ExprId) -> ItemId {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.resolve_method_target(id)
}

pub trait DriverContext<'ast> {
    fn lint_level_at(&'ast self, lint: &'static Lint, node: EmissionNode) -> Level;
    fn emit_diag(&'ast self, diag: &Diagnostic<'_, 'ast>);

    fn item(&'ast self, api_id: ItemId) -> Option<ItemKind<'ast>>;
    fn body(&'ast self, api_id: BodyId) -> &'ast Body<'ast>;

    fn resolve_ty_ids(&'ast self, path: &str) -> &'ast [TyDefId];

    fn expr_ty(&'ast self, expr: ExprId) -> SemTyKind<'ast>;
    fn span(&'ast self, owner: SpanId) -> &'ast Span<'ast>;
    fn span_snippet(&'ast self, span: &Span<'ast>) -> Option<&'ast str>;
    fn symbol_str(&'ast self, api_id: SymbolId) -> &'ast str;
    fn resolve_method_target(&'ast self, id: ExprId) -> ItemId;
}
