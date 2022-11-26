use std::iter;

use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct SliceTy<'ast> {
    data: CommonTyData<'ast>,
    inner_ty: TyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SliceTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, inner_ty: TyKind<'ast>) -> Self {
        Self { data, inner_ty }
    }
}

super::impl_ty_data!(SliceTy<'ast>, Slice);

impl<'ast> SliceTy<'ast> {
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}

impl<'ast> std::fmt::Debug for SliceTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(iter::once(self.inner_ty())).finish()
    }
}
