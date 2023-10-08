use crate::{
    ast::generic::Lifetime,
    common::{Abi, Mutability, Safety, SpanId},
    ffi::{FfiOption, FfiSlice},
    span::Ident,
};

use super::{CommonSynTyData, TyKind};

/// The syntactic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)

#[repr(C)]
#[derive(Debug)]
pub struct RefTy<'ast> {
    data: CommonSynTyData<'ast>,
    lifetime: FfiOption<Lifetime<'ast>>,
    mutability: Mutability,
    inner_ty: TyKind<'ast>,
}

impl<'ast> RefTy<'ast> {
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.get().is_some()
    }

    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(RefTy<'ast>, Ref);

#[cfg(feature = "driver-api")]
impl<'ast> RefTy<'ast> {
    pub fn new(
        data: CommonSynTyData<'ast>,
        lifetime: Option<Lifetime<'ast>>,
        mutability: Mutability,
        inner_ty: TyKind<'ast>,
    ) -> Self {
        Self {
            data,
            lifetime: lifetime.into(),
            mutability,
            inner_ty,
        }
    }
}

/// The syntactic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
pub struct RawPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    mutability: Mutability,
    inner_ty: TyKind<'ast>,
}

impl<'ast> RawPtrTy<'ast> {
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(RawPtrTy<'ast>, RawPtr);

#[cfg(feature = "driver-api")]
impl<'ast> RawPtrTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, mutability: Mutability, inner_ty: TyKind<'ast>) -> Self {
        Self {
            data,
            mutability,
            inner_ty,
        }
    }
}

/// The syntactic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct FnPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    safety: Safety,
    abi: Abi,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    params: FfiSlice<'ast, FnTyParameter<'ast>>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    return_ty: FfiOption<TyKind<'ast>>,
    // FIXME: Add `for<'a>` bound
}

impl<'ast> FnPtrTy<'ast> {
    /// Returns the [`Safety`] of this callable.
    ///
    /// Use this to check if the function is `unsafe`.
    pub fn safety(&self) -> Safety {
        self.safety
    }

    /// Returns the [`Abi`] of the callable.
    pub fn abi(&self) -> Abi {
        self.abi
    }

    /// Returns the [`FnTyParameter`]s this function pointer accepts.
    pub fn params(&self) -> &[FnTyParameter<'ast>] {
        self.params.get()
    }

    /// The return type of this function pointer, if specified.
    pub fn return_ty(&self) -> Option<&TyKind<'ast>> {
        self.return_ty.get()
    }
}

super::impl_ty_data!(FnPtrTy<'ast>, FnPtr);

/// A parameter for the [`FnPtrTy`].
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct FnTyParameter<'ast> {
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    ident: FfiOption<Ident<'ast>>,
    span: SpanId,
    ty: TyKind<'ast>,
}

impl<'ast> FnTyParameter<'ast> {
    /// Returns the [`Ident`] of the parameter, if specified.
    pub fn ident(&self) -> Option<&Ident<'ast>> {
        self.ident.get()
    }

    /// The syntactic type of this parameter.
    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }
}

crate::span::impl_has_span_via_field!(FnTyParameter<'ast>);
