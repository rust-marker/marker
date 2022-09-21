use crate::ast::Mutability;

use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(Debug)]
pub struct RawPtrTy<'ast> {
    data: CommonTyData<'ast>,
    mutability: Mutability,
    inner_ty: TyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> RawPtrTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, mutability: Mutability, inner_ty: TyKind<'ast>) -> Self {
        Self {
            data,
            mutability,
            inner_ty,
        }
    }
}

super::impl_ty_data!(RawPtrTy<'ast>, RawPtr);

impl<'ast> RawPtrTy<'ast> {
    pub fn is_mut(&self) -> bool {
        matches!(self.mutability, Mutability::Mut)
    }

    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}
