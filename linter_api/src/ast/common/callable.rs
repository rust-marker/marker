use crate::{
    ast::ty::TyKind,
    context::AstContext,
    ffi::{FfiOption, FfiSlice},
};

use super::{Abi, Span, SpanId, SymbolId};

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

    /// Returns `true`, if this callable is marked as extern. Bare functions
    /// only use the `extern` keyword to specify the ABI. These will currently
    /// still return `false` even if the keyword is present. In those cases,
    /// please refer to the ABI instead.
    ///
    /// Defaults to `false` if unspecified.
    fn is_extern(&self) -> bool;

    /// Returns the [`Abi`] of the callable, if specified.
    fn abi(&self) -> Abi;

    /// Returns `true`, if this callable has a specified `self` argument. The
    /// type of `self` can be retrieved from the first element of
    /// [`Callable::params()`].
    fn has_self(&self) -> bool;

    /// Returns the parameters, that this callable accepts. The `self` argument
    /// of methods, will be the first element of this slice. Use
    /// [`Callable::has_self`] to determine if the first argument is `self`.
    fn params(&self) -> &[Parameter<'ast>];

    /// Returns the return type, if specified.
    fn return_ty(&self) -> Option<&TyKind<'ast>>;
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Parameter<'ast> {
    cx: &'ast AstContext<'ast>,
    name: FfiOption<SymbolId>,
    ty: FfiOption<TyKind<'ast>>,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> Parameter<'ast> {
    pub fn new(
        cx: &'ast AstContext<'ast>,
        name: Option<SymbolId>,
        ty: Option<TyKind<'ast>>,
        span: Option<SpanId>,
    ) -> Self {
        Self {
            cx,
            name: name.into(),
            ty: ty.into(),
            span: span.into(),
        }
    }
}

impl<'ast> Parameter<'ast> {
    // Function items actually use patterns and not names. Patterns are not yet
    // implemented though. A pattern should be good enough for now.
    pub fn name(&self) -> Option<String> {
        self.name.get().map(|sym| self.cx.symbol_str(*sym))
    }

    pub fn ty(&self) -> Option<TyKind<'ast>> {
        self.ty.get().copied()
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().copied().map(|span| self.cx.get_span(span))
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct CallableData<'ast> {
    pub(crate) is_const: bool,
    pub(crate) is_async: bool,
    pub(crate) is_unsafe: bool,
    pub(crate) is_extern: bool,
    pub(crate) abi: Abi,
    pub(crate) has_self: bool,
    pub(crate) params: FfiSlice<'ast, Parameter<'ast>>,
    pub(crate) return_ty: FfiOption<TyKind<'ast>>,
}

#[cfg(feature = "driver-api")]
#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
impl<'ast> CallableData<'ast> {
    pub fn new(
        is_const: bool,
        is_async: bool,
        is_unsafe: bool,
        is_extern: bool,
        abi: Abi,
        has_self: bool,
        params: &'ast [Parameter<'ast>],
        return_ty: Option<TyKind<'ast>>,
    ) -> Self {
        Self {
            is_const,
            is_async,
            is_unsafe,
            is_extern,
            abi,
            has_self,
            params: params.into(),
            return_ty: return_ty.into(),
        }
    }
}

macro_rules! impl_callable_trait {
    ($self_ty:ty) => {
        impl<'ast> $crate::ast::common::Callable<'ast> for $self_ty {
            fn is_const(&self) -> bool {
                self.callable_data.is_const
            }
            fn is_async(&self) -> bool {
                self.callable_data.is_async
            }
            fn is_unsafe(&self) -> bool {
                self.callable_data.is_unsafe
            }
            fn is_extern(&self) -> bool {
                self.callable_data.is_extern
            }
            fn abi(&self) -> $crate::ast::common::Abi {
                self.callable_data.abi
            }
            fn has_self(&self) -> bool {
                self.callable_data.has_self
            }
            fn params(&self) -> &[$crate::ast::common::Parameter<'ast>] {
                self.callable_data.params.get()
            }
            fn return_ty(&self) -> Option<&$crate::ast::ty::TyKind<'ast>> {
                self.callable_data.return_ty.get()
            }
        }
    };
}
pub(crate) use impl_callable_trait;
