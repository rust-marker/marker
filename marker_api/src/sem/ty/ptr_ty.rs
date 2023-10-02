use crate::{
    ast::{Abi, Mutability, Safety},
    ffi::FfiSlice,
};

use super::SemTyKind;

/// The semantic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)
///
/// Note that the semantic representation doesn't contain lifetime information.
/// Marker currently doesn't support the analysis of lifetimes. Removing them
/// from the type also simplifies type comparisons.
#[repr(C)]
#[derive(Debug)]
pub struct SemRefTy<'ast> {
    mutability: Mutability,
    inner_ty: SemTyKind<'ast>,
}

impl<'ast> SemRefTy<'ast> {
    /// This returns the [`Mutability`] of the referenced type.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// This returns the inner [`SemTyKind`]
    pub fn inner_ty(&self) -> SemTyKind<'ast> {
        self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRefTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTyKind<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
pub struct SemRawPtrTy<'ast> {
    mutability: Mutability,
    inner_ty: SemTyKind<'ast>,
}

impl<'ast> SemRawPtrTy<'ast> {
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> SemTyKind<'ast> {
        self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRawPtrTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTyKind<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
#[repr(C)]
#[derive(Debug)]
pub struct SemFnPtrTy<'ast> {
    safety: Safety,
    abi: Abi,
    params: FfiSlice<'ast, SemTyKind<'ast>>,
    return_ty: SemTyKind<'ast>,
}

impl<'ast> SemFnPtrTy<'ast> {
    pub fn safety(&self) -> Safety {
        self.safety
    }

    pub fn abi(&self) -> Abi {
        self.abi
    }

    pub fn params(&self) -> &[SemTyKind<'ast>] {
        self.params.get()
    }

    pub fn return_ty(&self) -> SemTyKind<'ast> {
        self.return_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemFnPtrTy<'ast> {
    pub fn new(safety: Safety, abi: Abi, params: &'ast [SemTyKind<'ast>], return_ty: SemTyKind<'ast>) -> Self {
        Self {
            safety,
            abi,
            params: params.into(),
            return_ty,
        }
    }
}
