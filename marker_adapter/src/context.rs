// The lifetimes are destroyed by unsafe, but help with readability
#![allow(clippy::needless_lifetimes)]

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

// False positive because `EmissionNode` are non-exhaustive
#[allow(improper_ctypes_definitions)]
extern "C" fn lint_level_at(data: &(), lint: &'static Lint, node: EmissionNode) -> Level {
    unsafe { as_driver_cx(data) }.lint_level_at(lint, node)
}

extern "C" fn emit_diag<'a, 'ast>(data: &'ast (), diag: &Diagnostic<'a, 'ast>) {
    unsafe { as_driver_cx(data) }.emit_diag(diag);
}

// False positive because `ItemKind` is non-exhaustive
#[allow(improper_ctypes_definitions)]
extern "C" fn item<'ast>(data: &'ast (), id: ItemId) -> FfiOption<ItemKind<'ast>> {
    unsafe { as_driver_cx(data) }.item(id).into()
}

extern "C" fn body<'ast>(data: &'ast (), id: BodyId) -> &'ast Body<'ast> {
    unsafe { as_driver_cx(data) }.body(id)
}

extern "C" fn resolve_ty_ids<'ast>(data: &'ast (), path: ffi::FfiStr<'_>) -> ffi::FfiSlice<'ast, TyDefId> {
    unsafe { as_driver_cx(data) }.resolve_ty_ids((&path).into()).into()
}

// False positive because `SemTyKind` is non-exhaustive
#[allow(improper_ctypes_definitions)]
extern "C" fn expr_ty<'ast>(data: &'ast (), expr: ExprId) -> SemTyKind<'ast> {
    unsafe { as_driver_cx(data) }.expr_ty(expr)
}

extern "C" fn span<'ast>(data: &'ast (), span_id: SpanId) -> &'ast Span<'ast> {
    unsafe { as_driver_cx(data) }.span(span_id)
}

extern "C" fn span_snippet<'ast>(data: &'ast (), span: &Span<'ast>) -> ffi::FfiOption<ffi::FfiStr<'ast>> {
    unsafe { as_driver_cx(data) }.span_snippet(span).map(Into::into).into()
}

extern "C" fn symbol_str<'ast>(data: &'ast (), sym: SymbolId) -> ffi::FfiStr<'ast> {
    unsafe { as_driver_cx(data) }.symbol_str(sym).into()
}

extern "C" fn resolve_method_target(data: &(), id: ExprId) -> ItemId {
    unsafe { as_driver_cx(data) }.resolve_method_target(id)
}

/// # Safety
/// The `data` must be a valid pointer to a [`DriverContextWrapper`]
unsafe fn as_driver_cx<'ast>(data: &'ast ()) -> &'ast dyn DriverContext<'ast> {
    let wrapper = &*(data as *const ()).cast::<DriverContextWrapper>();
    wrapper.driver_cx
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
