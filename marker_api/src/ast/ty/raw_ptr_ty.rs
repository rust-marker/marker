use super::{CommonSynTyData, SynTyKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RawPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    is_mut: bool,
    inner_ty: SynTyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> RawPtrTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, is_mut: bool, inner_ty: SynTyKind<'ast>) -> Self {
        Self { data, is_mut, inner_ty }
    }
}

super::impl_ty_data!(RawPtrTy<'ast>, RawPtr);

impl<'ast> RawPtrTy<'ast> {
    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}
