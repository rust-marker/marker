use crate::ast::{generic::GenericArgs, TyDefId};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct UnionTy<'ast> {
    data: CommonTyData<'ast>,
    def_id: TyDefId,
    generic_args: GenericArgs<'ast>,
    // FIXME: Add representation/layout info like alignment, size, type
}

#[cfg(feature = "driver-api")]
impl<'ast> UnionTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, def_id: TyDefId, generic_args: GenericArgs<'ast>) -> Self {
        Self {
            data,
            def_id,
            generic_args,
        }
    }
}

super::impl_ty_data!(UnionTy<'ast>, Union);

impl<'ast> UnionTy<'ast> {
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

    pub fn generic_args(&self) -> &GenericArgs<'ast> {
        &self.generic_args
    }
}
