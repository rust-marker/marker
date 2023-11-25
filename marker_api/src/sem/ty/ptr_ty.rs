use crate::{
    common::{Abi, Mutability, Safety},
    ffi::FfiSlice,
};

use super::{CommonTyData, TyKind};

/// The semantic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)
///
/// Note that the semantic representation doesn't contain lifetime information.
/// Marker currently doesn't support the analysis of lifetimes. Removing them
/// from the type also simplifies type comparisons.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct RefTy<'ast> {
    data: CommonTyData<'ast>,
    mutability: Mutability,
    inner_ty: TyKind<'ast>,
}

impl<'ast> RefTy<'ast> {
    /// This returns the [`Mutability`] of the referenced type.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// This returns the inner [`TyKind`]
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(RefTy<'ast>, Ref);

/// The semantic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct RawPtrTy<'ast> {
    data: CommonTyData<'ast>,
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

/// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct FnPtrTy<'ast> {
    data: CommonTyData<'ast>,
    safety: Safety,
    abi: Abi,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    params: FfiSlice<'ast, TyKind<'ast>>,
    return_ty: TyKind<'ast>,
}

impl<'ast> FnPtrTy<'ast> {
    pub fn safety(&self) -> Safety {
        self.safety
    }

    pub fn abi(&self) -> Abi {
        self.abi
    }

    pub fn params(&self) -> &[TyKind<'ast>] {
        self.params.get()
    }

    pub fn return_ty(&self) -> TyKind<'ast> {
        self.return_ty
    }
}

super::impl_ty_data!(FnPtrTy<'ast>, FnPtr);
