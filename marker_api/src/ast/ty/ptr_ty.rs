use crate::{
    ast::{Abi, Mutability, Safety},
    ffi::FfiSlice,
};

use super::SemTy;

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
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemRefTy<'ast> {
    /// This returns the [`Mutability`] of the referenced type.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// This returns the inner [`SemTy`]
    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRefTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTy<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
pub struct SemRawPtrTy<'ast> {
    mutability: Mutability,
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemRawPtrTy<'ast> {
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRawPtrTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTy<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a function pointer, like
/// [`fn(u32) -> i32`](<https://doc.rust-lang.org/stable/std/keyword.fn.html>)
#[repr(C)]
#[derive(Debug)]
pub struct SemFnPtrTy<'ast> {
    safety: Safety,
    abi: Abi,
    params: FfiSlice<'ast, SemTy<'ast>>,
    return_ty: SemTy<'ast>,
}

impl<'ast> SemFnPtrTy<'ast> {
    pub fn safety(&self) -> Safety {
        self.safety
    }

    pub fn abi(&self) -> Abi {
        self.abi
    }

    pub fn params(&self) -> &[SemTy<'ast>] {
        self.params.get()
    }

    pub fn return_ty(&self) -> &SemTy<'ast> {
        &self.return_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemFnPtrTy<'ast> {
    pub fn new(safety: Safety, abi: Abi, params: &'ast [SemTy<'ast>], return_ty: SemTy<'ast>) -> Self {
        Self {
            safety,
            abi,
            params: params.into(),
            return_ty,
        }
    }
}
