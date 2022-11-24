use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RawPtrTy<'ast> {
    data: CommonTyData<'ast>,
    is_mut: bool,
    inner_ty: TyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> RawPtrTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, is_mut: bool, inner_ty: TyKind<'ast>) -> Self {
        Self { data, is_mut, inner_ty }
    }
}

super::impl_ty_data!(RawPtrTy<'ast>, RawPtr);

impl<'ast> RawPtrTy<'ast> {
    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}
