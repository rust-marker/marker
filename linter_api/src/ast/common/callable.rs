use crate::{
    ast::ty::TyKind,
    ffi::{FfiOption, FfiSlice},
};

use super::{Abi, SpanId, SymbolId};

/// This trait provides informations about callable items and types. Some
/// properties might not be available for every callable object. In those
/// cases the default value will be returned.
pub trait Callable<'ast> {
    /// Returns `true`, if this callable is `const`.
    ///
    /// Defaults to `false` if unspecified.
    fn is_const(&self) -> bool;

    /// Returns `true`, if this callable is `async`.
    ///
    /// Defaults to `false` if unspecified.
    fn is_async(&self) -> bool;

    /// Returns `true`, if this callable is marked as `unsafe`.
    ///
    /// Defaults to `false` if unspecified. Extern functions will
    /// also return `false` by default, even if they require `unsafe`
    /// by default.
    fn is_unsafe(&self) -> bool;

    /// Returns `true`, if this callable is marked as extern.
    ///
    /// Defaults to `false` if unspecified.
    fn is_extern(&self) -> bool;

    /// Returns the [`Abi`] of the callable, if specified.
    fn abi(&self) -> Option<Abi>;

    /// Returns `true`, if this callable has a specified `self` argument. The
    /// type of `self` can be retrieved from the first element of
    /// [`Callable::params()`].
    fn has_self(&self) -> bool;

    /// Returns the parameters, that this callable accepts. The `self` argument
    /// of methods, will be the first element of this slice. Use
    /// [`Callable::has_self`] to determine if the first argument is `self`.
    fn params(&self) -> &[&Parameter<'ast>];

    /// Returns the return type, if specified.
    fn return_ty(&self) -> Option<&TyKind<'ast>>;
}

#[repr(C)]
#[derive(Debug)]
pub struct Parameter<'ast> {
    name: FfiOption<SymbolId>,
    ty: FfiOption<TyKind<'ast>>,
    span: FfiOption<SpanId>,
}

impl<'ast> Parameter<'ast> {
    // Function items actually use patterns and not names. Patterns are not yet
    // implemented though. A pattern should be good enough for now.
    pub fn name(&self) -> Option<SymbolId> {
        self.name.get().copied()
    }

    pub fn ty(&self) -> Option<TyKind<'ast>> {
        self.ty.get().copied()
    }

    pub fn span(&self) -> Option<SpanId> {
        self.span.get().copied()
    }
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct CallableData<'ast> {
    is_const: bool,
    is_async: bool,
    is_unsafe: bool,
    is_extern: bool,
    abi: FfiOption<Abi>,
    has_self: bool,
    params: FfiSlice<'ast, &'ast Parameter<'ast>>,
    return_ty: FfiOption<TyKind<'ast>>,
}
