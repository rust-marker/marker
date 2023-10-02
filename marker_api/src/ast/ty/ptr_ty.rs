use crate::{
    ast::{generic::Lifetime, Abi, Mutability, Safety},
    common::SpanId,
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
    private::Sealed,
    span::{HasSpan, Ident},
};

use super::{CommonSynTyData, SynTyKind};

/// The syntactic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)

#[repr(C)]
#[derive(Debug)]
pub struct SynRefTy<'ast> {
    data: CommonSynTyData<'ast>,
    lifetime: FfiOption<Lifetime<'ast>>,
    mutability: Mutability,
    inner_ty: SynTyKind<'ast>,
}

impl<'ast> SynRefTy<'ast> {
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.get().is_some()
    }

    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(SynRefTy<'ast>, Ref);

#[cfg(feature = "driver-api")]
impl<'ast> SynRefTy<'ast> {
    pub fn new(
        data: CommonSynTyData<'ast>,
        lifetime: Option<Lifetime<'ast>>,
        mutability: Mutability,
        inner_ty: SynTyKind<'ast>,
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
pub struct SynRawPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    mutability: Mutability,
    inner_ty: SynTyKind<'ast>,
}

impl<'ast> SynRawPtrTy<'ast> {
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(SynRawPtrTy<'ast>, RawPtr);

#[cfg(feature = "driver-api")]
impl<'ast> SynRawPtrTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, mutability: Mutability, inner_ty: SynTyKind<'ast>) -> Self {
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
pub struct SynFnPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    safety: Safety,
    abi: Abi,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    params: FfiSlice<'ast, FnTyParameter<'ast>>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    return_ty: FfiOption<SynTyKind<'ast>>,
    // FIXME: Add `for<'a>` bound
}

impl<'ast> SynFnPtrTy<'ast> {
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
    pub fn return_ty(&self) -> Option<&SynTyKind<'ast>> {
        self.return_ty.get()
    }
}

super::impl_ty_data!(SynFnPtrTy<'ast>, FnPtr);

/// A parameter for the [`SynFnPtrTy`].
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct FnTyParameter<'ast> {
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    ident: FfiOption<Ident<'ast>>,
    span: SpanId,
    ty: SynTyKind<'ast>,
}

impl<'ast> FnTyParameter<'ast> {
    /// Returns the [`Ident`] of the parameter, if specified.
    pub fn ident(&self) -> Option<&Ident<'ast>> {
        self.ident.get()
    }

    /// The syntactic type of this parameter.
    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }
}

impl Sealed for FnTyParameter<'_> {}
impl<'ast> HasSpan<'ast> for FnTyParameter<'ast> {
    fn span(&self) -> &crate::span::Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }
}
