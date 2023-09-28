use std::fmt::Debug;

use crate::{
    ast::ty::SynTyKind,
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
    private::Sealed,
    span::Span,
};

use super::{Abi, Constness, Safety, SpanId, SymbolId, Syncness};

/// This trait provides information about callable items and types. Some
/// properties might not be available for every callable object. In these
/// cases the default value will be returned.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait CallableData<'ast>: Debug + Sealed {
    /// Returns the [`Constness`] of this callable
    fn constness(&self) -> Constness;

    /// Returns the [`Syncness`] of this callable.
    ///
    /// Use this to check if the function is async.
    fn syncness(&self) -> Syncness;

    /// Returns the [`Safety`] of this callable.
    ///
    /// Use this to check if the function is unsafe.
    fn safety(&self) -> Safety;

    /// Returns `true`, if this callable is marked as `extern`. Bare functions
    /// only use the `extern` keyword to specify the ABI. These will currently
    /// still return `false` even if the keyword is present. In those cases,
    /// please refer to the [`abi()`](`Self::abi`) instead.
    ///
    /// Defaults to `false` if unspecified.
    fn is_extern(&self) -> bool;

    /// Returns the [`Abi`] of the callable.
    fn abi(&self) -> Abi;

    /// Returns `true`, if this callable has a specified `self` argument. The
    /// type of `self` can be retrieved from the first element of
    /// [`params()`](`Self::params`).
    fn has_self(&self) -> bool;

    /// Returns the parameters, that this callable accepts. The `self` argument
    /// of methods, will be the first element of this slice. Use
    /// [`has_self()`](`Self::has_self`) to determine if the first argument is `self`.
    fn params(&self) -> &[Parameter<'ast>];

    /// The return type of this callable, if specified.
    fn return_ty(&self) -> Option<&SynTyKind<'ast>>;
}

#[repr(C)]
#[derive(Debug)]
pub struct Parameter<'ast> {
    name: FfiOption<SymbolId>,
    ty: FfiOption<SynTyKind<'ast>>,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> Parameter<'ast> {
    pub fn new(name: Option<SymbolId>, ty: Option<SynTyKind<'ast>>, span: Option<SpanId>) -> Self {
        Self {
            name: name.into(),
            ty: ty.into(),
            span: span.into(),
        }
    }
}

impl<'ast> Parameter<'ast> {
    /// FIXME(xFrednet): This function returns the name of the parameter, if it's a
    /// single value. It should be replaced with a pattern instead, see rust-marker/marker#181
    pub fn name(&self) -> Option<&str> {
        self.name.get().map(|sym| with_cx(self, |cx| cx.symbol_str(*sym)))
    }

    pub fn ty(&self) -> Option<SynTyKind<'ast>> {
        self.ty.copy()
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().copied().map(|span| with_cx(self, |cx| cx.span(span)))
    }
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct CommonCallableData<'ast> {
    pub(crate) constness: Constness,
    pub(crate) syncness: Syncness,
    pub(crate) safety: Safety,
    pub(crate) is_extern: bool,
    pub(crate) abi: Abi,
    pub(crate) has_self: bool,
    pub(crate) params: FfiSlice<'ast, Parameter<'ast>>,
    pub(crate) return_ty: FfiOption<SynTyKind<'ast>>,
}

#[cfg(feature = "driver-api")]
#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
impl<'ast> CommonCallableData<'ast> {
    pub fn new(
        constness: Constness,
        syncness: Syncness,
        safety: Safety,
        is_extern: bool,
        abi: Abi,
        has_self: bool,
        params: &'ast [Parameter<'ast>],
        return_ty: Option<SynTyKind<'ast>>,
    ) -> Self {
        Self {
            constness,
            syncness,
            safety,
            is_extern,
            abi,
            has_self,
            params: params.into(),
            return_ty: return_ty.into(),
        }
    }
}

/// This macro automatically implements the [`CallableData`] trait for structs that
/// have a `callable_data` field.
macro_rules! impl_callable_data_trait {
    ($self_ty:ty) => {
        impl<'ast> $crate::ast::common::CallableData<'ast> for $self_ty {
            fn constness(&self) -> $crate::ast::Constness {
                self.callable_data.constness
            }
            fn syncness(&self) -> $crate::ast::Syncness {
                self.callable_data.syncness
            }
            fn safety(&self) -> $crate::ast::Safety {
                self.callable_data.safety
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
            fn return_ty(&self) -> Option<&$crate::ast::ty::SynTyKind<'ast>> {
                self.callable_data.return_ty.get()
            }
        }
    };
}
pub(crate) use impl_callable_data_trait;
