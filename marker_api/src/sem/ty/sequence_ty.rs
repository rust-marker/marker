use crate::{ffi::FfiSlice, sem::ConstValue};

use super::TyKind;

/// The semantic representation of a tuple type like [`()`](prim@tuple) or [`(T, U)`](prim@tuple)
#[repr(C)]
#[derive(Debug)]
pub struct TupleTy<'ast> {
    types: FfiSlice<'ast, TyKind<'ast>>,
}

impl<'ast> TupleTy<'ast> {
    pub fn types(&self) -> &[TyKind<'ast>] {
        self.types.as_slice()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TupleTy<'ast> {
    pub fn new(types: &'ast [TyKind<'ast>]) -> Self {
        Self { types: types.into() }
    }
}

impl<'ast> std::fmt::Display for TupleTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_tuple("");

        for entry in self.types.as_slice() {
            f.field(entry);
        }

        f.finish()
    }
}

/// The semantic representation of a variable length slice like [`[T]`](prim@slice)
#[repr(C)]
pub struct SliceTy<'ast> {
    inner_ty: TyKind<'ast>,
}

impl<'ast> SliceTy<'ast> {
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SliceTy<'ast> {
    pub fn new(inner_ty: TyKind<'ast>) -> Self {
        Self { inner_ty }
    }
}

impl<'ast> std::fmt::Debug for SliceTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}

/// The semantic representation of an array with a known size like: [`[T; N]`](prim@array)
#[repr(C)]
#[derive(Debug)]
pub struct ArrayTy<'ast> {
    inner_ty: TyKind<'ast>,
    len: ConstValue<'ast>,
}

impl<'ast> ArrayTy<'ast> {
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }

    pub fn len(&self) -> &ConstValue<'ast> {
        &self.len
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ArrayTy<'ast> {
    pub fn new(inner_ty: TyKind<'ast>, len: ConstValue<'ast>) -> Self {
        Self { inner_ty, len }
    }
}

impl<'ast> std::fmt::Display for ArrayTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Add length expression
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}
