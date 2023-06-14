use std::iter;

use super::{CommonSynTyData, SynTyKind};

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct SliceTy<'ast> {
    data: CommonSynTyData<'ast>,
    inner_ty: SynTyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SliceTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, inner_ty: SynTyKind<'ast>) -> Self {
        Self { data, inner_ty }
    }
}

super::impl_ty_data!(SliceTy<'ast>, Slice);

impl<'ast> SliceTy<'ast> {
    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}

impl<'ast> std::fmt::Debug for SliceTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(iter::once(self.inner_ty())).finish()
    }
}
