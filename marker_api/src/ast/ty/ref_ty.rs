use crate::{ast::generic::Lifetime, ffi::FfiOption};

use super::{CommonSynTyData, SynTyKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RefTy<'ast> {
    data: CommonSynTyData<'ast>,
    lifetime: FfiOption<Lifetime<'ast>>,
    is_mut: bool,
    inner_ty: SynTyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> RefTy<'ast> {
    pub fn new(
        data: CommonSynTyData<'ast>,
        lifetime: Option<Lifetime<'ast>>,
        is_mut: bool,
        inner_ty: SynTyKind<'ast>,
    ) -> Self {
        Self {
            data,
            lifetime: lifetime.into(),
            is_mut,
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
        self.is_mut
    }

    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}
