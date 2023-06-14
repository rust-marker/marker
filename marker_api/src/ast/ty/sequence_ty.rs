use crate::ffi::FfiSlice;

use super::SemTyKind;

/// The semantic representation of a tuple type like [`()`](prim@tuple) or [`(T, U)`](prim@tuple)
#[repr(C)]
pub struct SemTupleTy<'ast> {
    types: FfiSlice<'ast, SemTyKind<'ast>>,
}

impl<'ast> SemTupleTy<'ast> {
    pub fn types(&self) -> &[SemTyKind<'ast>] {
        self.types.as_slice()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTupleTy<'ast> {
    pub fn new(types: &'ast [SemTyKind<'ast>]) -> Self {
        Self { types: types.into() }
    }
}

impl<'ast> std::fmt::Debug for SemTupleTy<'ast> {
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
pub struct SemSliceTy<'ast> {
    inner_ty: SemTyKind<'ast>,
}

impl<'ast> SemSliceTy<'ast> {
    pub fn inner_ty(&self) -> SemTyKind<'ast> {
        self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemSliceTy<'ast> {
    pub fn new(inner_ty: SemTyKind<'ast>) -> Self {
        Self { inner_ty }
    }
}

impl<'ast> std::fmt::Debug for SemSliceTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}

/// The semantic representation of an array with a known size like: [`[T; N]`](prim@array)
#[repr(C)]
pub struct SemArrayTy<'ast> {
    inner_ty: SemTyKind<'ast>,
}

impl<'ast> SemArrayTy<'ast> {
    pub fn inner_ty(&self) -> SemTyKind<'ast> {
        self.inner_ty
    }

    pub fn len(&self) {
        // FIXME: Add length expression
        unimplemented!()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemArrayTy<'ast> {
    pub fn new(inner_ty: SemTyKind<'ast>) -> Self {
        Self { inner_ty }
    }
}

impl<'ast> std::fmt::Debug for SemArrayTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Add length expression
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}
