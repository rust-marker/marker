//! The [`marker_api`] crate is designed to be driver-independent and lightweight.
//! Communication from the lint-crate to the driver is done via structs inside the
//! [`marker_api::context`] module. For example, [`marker_api::MarkerContext`].
//!
//! The communication needs to be ABI safe, since it goes over an FFI function
//! boundary. For this reason, it's not (directly) possible to use `dyn Trait`objects.
//! Marker instead uses `*Callback` structs, that contain function pointers, which can
//! be filled by the driver. This is like a `dyn Trait` object, without all the nice
//! syntax and build in compiler support.
//!
//! These `*Callbacks` structs need to be filled and some ABI-specific transformations
//! have to be performed. This module hides all the gore behind some structs and
//! traits, which can be implemented by the specific driver. The following is an
//! explanation of this structure and some naming conventions. The explanation will use
//! an imaginary `Magic` struct, to illustrate the structure:
//!
//! * The `marker_api` crate defines the `Magic` struct with an interface for external and internal
//!   consumers.
//!
//!    It also defines the `MagicCallbacks` struct, which stores function pointers.
//!    All `*Callbacks` structs have a special `data: &'ast MarkerContextData` field that will be
//!    passed to all stored function pointers, as the first argument. All function
//!    pointers are marked as `extern "C"` and need to use FFI safe types. Lifetimes
//!    are allowed by Rust, but not enforced over FFI bounds.
//!
//! * The `marker_adapter` crate defines a `MagicDriver` trait, that has all the functions, needed
//!   to provide a backend for the `Magic` struct.
//!
//!    It also defines the `MagicWrapper` struct, that has a `driver: &'ast dyn
//!    MagicDriver` field. This struct will be used to fill the `MagicCallbacks.data`
//!    field. Wrapping it in a separate `*Wrapper` struct makes the `&'ast *Wrapper`
//!    pointer a thin pointer and cleans up the interface.
//!
//!    The `marker_adapter` module containing the `MagicWrapper` struct defines a
//!    bunch of `extern "C"` functions, which build the counterpart to the `extern
//!    "C"` functions inside `MagicCallbacks`. These functions cast the `data`
//!    argument into the `&'ast MagicWrapper` instance it originated from and calls
//!    the corresponding trait function from the `driver: &'ast dyn MagicDriver` field
//!    stored inside the `Wrapper` struct. The `extern "C"` functions are also
//!    responsible for converting all types into FFI safe types and back.
//!
//! * The driver simply implements the `MagicDriver` trait and instantiates `MagicWrapper`. This
//!   wrapper instance is then used to fill `MagicCallbacks` and instantiate the `Magic` struct from
//!   `marker_api`.
//!
//! ---
//!
//! Short Q&A with @xFrednet
//!
//! * **Isn't there a simpler way?**
//!
//!     Most likely... Or better say hopefully there is.
//!
//!     However, I couldn't find one. Most libraries I found either didn't support these
//!     types of callbacks, were experimental, or are unsound.
//!
//!     I also have high hopes for Rust adding some cross crate boundary communication
//!     support some time in the future. But there is no point in waiting on this. There
//!     are lints which need to be written, and we can always replace this infrastructure
//!     later.
//!
//! * **Isn't there a simpler way to make the types FFI safe?**
//!
//!     Not really. There are some tools to generate ABI safe types, but they are geared
//!     towards C/C++ consumers. As a result, they usually lose lifetime information,
//!     which we want to keep since both sides are using Rust.
//!
//!     There are also crates that rely on serialization. A nice and simple solution, but
//!     not feasible for an entire AST over multiple FFI boundaries for every lint crate.
//!
//! * **Do you use code generation for all this infrastructure?**
//!
//!     I would love to! However, I haven't found the time to implement this. Normal macro
//!     rules are basically out of the question due to all the required FFI type
//!     transformation.
//!
//!     If anyone is interested in implementing this, I would be grateful!!! See
//!     rust-marker/marker#122
//!
//! * **Is this implementation even safe and sound?**
//!
//!     Theoretically speaking? From my understanding? Yes, it is, assuming that both sides
//!     reconstruct the lifetimes correctly.
//!
//!     Practically speaking? It wouldn't surprise me if there were several bugs. So far,
//!     it has been working suspiciously well, but I won't complain.

// The lifetimes are destroyed by unsafe, but help with readability
#![allow(clippy::needless_lifetimes)]

mod map;
pub use map::*;

use marker_api::{
    common::{ExpnId, ExprId, SpanId, SymbolId},
    context::{MarkerContextCallbacks, MarkerContextData},
    diagnostic::Diagnostic,
    ffi::{self, FfiOption},
    prelude::*,
    span::{ExpnInfo, FileInfo, FilePos, SpanPos, SpanSource},
};

/// ### Safety
///
/// `&dyn` objects are theoretically not FFI safe since their type layout can
/// change and calling functions on them would require a stable ABI which Rust
/// doesn't provide.
///
/// In this case, the [`MarkerContextWrapper`] will be passed as a `*const ()`
/// pointer to [`MarkerContextCallbacks`] which will do nothing with this data other
/// than giving it back to functions declared in this module. Since the `&dyn`
/// object is created, only used here and everything is compiled during the same
/// compiler run, it should be safe to use `&dyn`.
#[repr(C)]
pub struct MarkerContextWrapper<'ast> {
    driver: &'ast dyn MarkerContextDriver<'ast>,
}

impl<'ast> MarkerContextWrapper<'ast> {
    pub fn new(driver: &'ast dyn MarkerContextDriver<'ast>) -> Self {
        Self { driver }
    }

    #[must_use]
    pub fn create_callbacks(&'ast self) -> MarkerContextCallbacks<'ast> {
        MarkerContextCallbacks {
            data: unsafe { &*(self as *const MarkerContextWrapper).cast::<MarkerContextData>() },
            emit_diag,
            resolve_ty_ids,
            expr_ty,
            span,
            span_snippet,
            span_source,
            span_pos_to_file_loc,
            span_expn_info,
            symbol_str,
            resolve_method_target,
        }
    }
}

pub trait MarkerContextDriver<'ast> {
    fn emit_diag(&'ast self, diag: &Diagnostic<'_, 'ast>);

    fn resolve_ty_ids(&'ast self, path: &str) -> &'ast [TyDefId];

    fn expr_ty(&'ast self, expr: ExprId) -> marker_api::sem::ty::TyKind<'ast>;
    fn span(&'ast self, owner: SpanId) -> &'ast Span<'ast>;
    fn span_snippet(&'ast self, span: &Span<'_>) -> Option<&'ast str>;
    fn span_source(&'ast self, span: &Span<'_>) -> SpanSource<'ast>;
    fn span_expn_info(&'ast self, expn_id: ExpnId) -> Option<&'ast ExpnInfo<'ast>>;
    fn span_pos_to_file_loc(&'ast self, file: &FileInfo<'ast>, pos: SpanPos) -> Option<FilePos<'ast>>;
    fn symbol_str(&'ast self, api_id: SymbolId) -> &'ast str;
    fn resolve_method_target(&'ast self, id: ExprId) -> ItemId;
}

extern "C" fn emit_diag<'a, 'ast>(data: &'ast MarkerContextData, diag: &Diagnostic<'a, 'ast>) {
    unsafe { as_driver(data) }.emit_diag(diag);
}

extern "C" fn resolve_ty_ids<'ast>(
    data: &'ast MarkerContextData,
    path: ffi::FfiStr<'_>,
) -> ffi::FfiSlice<'ast, TyDefId> {
    unsafe { as_driver(data) }.resolve_ty_ids((&path).into()).into()
}

// False positive because `SemTyKind` is non-exhaustive
#[allow(improper_ctypes_definitions)]
extern "C" fn expr_ty<'ast>(data: &'ast MarkerContextData, expr: ExprId) -> marker_api::sem::ty::TyKind<'ast> {
    unsafe { as_driver(data) }.expr_ty(expr)
}

extern "C" fn span<'ast>(data: &'ast MarkerContextData, span_id: SpanId) -> &'ast Span<'ast> {
    unsafe { as_driver(data) }.span(span_id)
}

extern "C" fn span_snippet<'ast>(
    data: &'ast MarkerContextData,
    span: &Span<'ast>,
) -> ffi::FfiOption<ffi::FfiStr<'ast>> {
    unsafe { as_driver(data) }.span_snippet(span).map(Into::into).into()
}

// False positive because `SpanSource` is non-exhaustive
#[allow(improper_ctypes_definitions)]
extern "C" fn span_source<'ast>(data: &'ast MarkerContextData, span: &Span<'_>) -> SpanSource<'ast> {
    unsafe { as_driver(data) }.span_source(span)
}

extern "C" fn span_pos_to_file_loc<'ast>(
    data: &'ast MarkerContextData,
    file: &FileInfo<'ast>,
    pos: SpanPos,
) -> ffi::FfiOption<FilePos<'ast>> {
    unsafe { as_driver(data) }.span_pos_to_file_loc(file, pos).into()
}

extern "C" fn span_expn_info<'ast>(data: &'ast MarkerContextData, expn_id: ExpnId) -> FfiOption<&'ast ExpnInfo<'ast>> {
    unsafe { as_driver(data) }.span_expn_info(expn_id).into()
}

extern "C" fn symbol_str<'ast>(data: &'ast MarkerContextData, sym: SymbolId) -> ffi::FfiStr<'ast> {
    unsafe { as_driver(data) }.symbol_str(sym).into()
}

extern "C" fn resolve_method_target<'ast>(data: &'ast MarkerContextData, id: ExprId) -> ItemId {
    unsafe { as_driver(data) }.resolve_method_target(id)
}

/// # Safety
/// The `data` must be a valid pointer to a [`MarkerContextWrapper`]
unsafe fn as_driver<'ast>(data: &'ast MarkerContextData) -> &'ast dyn MarkerContextDriver<'ast> {
    let wrapper = &*(data as *const MarkerContextData).cast::<MarkerContextWrapper>();
    wrapper.driver
}
