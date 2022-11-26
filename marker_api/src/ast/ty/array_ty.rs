use std::iter;

use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct ArrayTy<'ast> {
    data: CommonTyData<'ast>,
    inner_ty: TyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> ArrayTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, inner_ty: TyKind<'ast>) -> Self {
        Self { data, inner_ty }
    }
}

super::impl_ty_data!(ArrayTy<'ast>, Array);

impl<'ast> ArrayTy<'ast> {
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }

    pub fn len(&self) {
        // FIXME: Return expression
        unimplemented!()
    }
}

impl<'ast> std::fmt::Debug for ArrayTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Add return expression
        f.debug_list().entries(iter::once(self.inner_ty())).finish()
    }
}
