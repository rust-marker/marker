use crate::ffi::FfiSlice;

use super::SemTy;

#[repr(C)]
pub struct SemTupleTy<'ast> {
    types: FfiSlice<'ast, SemTy<'ast>>,
}

impl<'ast> SemTupleTy<'ast> {
    pub fn types(&self) -> &[SemTy<'ast>] {
        self.types.as_slice()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTupleTy<'ast> {
    pub fn new(types: &'ast [SemTy<'ast>]) -> Self {
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

#[repr(C)]
pub struct SemSliceTy<'ast> {
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemSliceTy<'ast> {
    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemSliceTy<'ast> {
    pub fn new(inner_ty: SemTy<'ast>) -> Self {
        Self { inner_ty }
    }
}

impl<'ast> std::fmt::Debug for SemSliceTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}

#[repr(C)]
pub struct SemArrayTy<'ast> {
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemArrayTy<'ast> {
    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }

    pub fn len(&self) {
        // FIXME: Add length expression
        unimplemented!()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemArrayTy<'ast> {
    pub fn new(inner_ty: SemTy<'ast>) -> Self {
        Self { inner_ty }
    }
}

impl<'ast> std::fmt::Debug for SemArrayTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Add length expression
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}
