use crate::{
    ast::{generic::SemLifetime, Mutability},
    ffi::FfiOption,
};

use super::SemTy;

#[repr(C)]
#[derive(Debug)]
pub struct SemRefTy<'ast> {
    lifetime: FfiOption<SemLifetime<'ast>>,
    mutability: Mutability,
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemRefTy<'ast> {
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.get().is_some()
    }

    pub fn lifetime(&self) -> Option<&SemLifetime<'ast>> {
        self.lifetime.get()
    }

    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRefTy<'ast> {
    pub fn new(lifetime: Option<SemLifetime<'ast>>, mutability: Mutability, inner_ty: SemTy<'ast>) -> Self {
        Self {
            lifetime: lifetime.into(),
            mutability,
            inner_ty,
        }
    }
}

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
