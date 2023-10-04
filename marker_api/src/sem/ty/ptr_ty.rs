use crate::{
    common::{Abi, Mutability, Safety},
    ffi::FfiSlice,
};

use super::TyKind;

/// The semantic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)
///
/// Note that the semantic representation doesn't contain lifetime information.
/// Marker currently doesn't support the analysis of lifetimes. Removing them
/// from the type also simplifies type comparisons.
#[repr(C)]
#[derive(Debug)]
pub struct RefTy<'ast> {
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

#[cfg(feature = "driver-api")]
impl<'ast> RefTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: TyKind<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
pub struct RawPtrTy<'ast> {
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

#[cfg(feature = "driver-api")]
impl<'ast> RawPtrTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: TyKind<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
#[repr(C)]
#[derive(Debug)]
pub struct FnPtrTy<'ast> {
    safety: Safety,
    abi: Abi,
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

#[cfg(feature = "driver-api")]
impl<'ast> FnPtrTy<'ast> {
    pub fn new(safety: Safety, abi: Abi, params: &'ast [TyKind<'ast>], return_ty: TyKind<'ast>) -> Self {
        Self {
            safety,
            abi,
            params: params.into(),
            return_ty,
        }
    }
}
