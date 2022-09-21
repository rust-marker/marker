use crate::{
    ast::{Lifetime, Mutability},
    ffi::FfiOption,
};

use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(Debug)]
pub struct RefTy<'ast> {
    data: CommonTyData<'ast>,
    lifetime: FfiOption<Lifetime<'ast>>,
    mutability: Mutability,
    inner_ty: TyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> RefTy<'ast> {
    pub fn new(
        data: CommonTyData<'ast>,
        lifetime: FfiOption<Lifetime<'ast>>,
        mutability: Mutability,
        inner_ty: TyKind<'ast>,
    ) -> Self {
        Self {
            data,
            lifetime,
            mutability,
            inner_ty,
        }
    }
}

super::impl_ty_data!(RefTy<'ast>, Ref);

impl<'ast> RefTy<'ast> {
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.get().is_some()
    }

    pub fn is_mut(&self) -> bool {
        matches!(self.mutability, Mutability::Mut)
    }

    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}
