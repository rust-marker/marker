use crate::ast::{generic::GenericArgs, TyDefId};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StructTy<'ast> {
    data: CommonTyData<'ast>,
    def_id: TyDefId,
    generic_args: GenericArgs<'ast>,
    is_non_exhaustive: bool,
    // FIXME: Add representation/layout info like alignment, size, type
}

#[cfg(feature = "driver-api")]
impl<'ast> StructTy<'ast> {
    pub fn new(
        data: CommonTyData<'ast>,
        def_id: TyDefId,
        generic_args: GenericArgs<'ast>,
        is_non_exhaustive: bool,
    ) -> Self {
        Self {
            data,
            def_id,
            generic_args,
            is_non_exhaustive,
        }
    }
}

super::impl_ty_data!(StructTy<'ast>, Struct);

impl<'ast> StructTy<'ast> {
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

    pub fn generic_args(&self) -> &GenericArgs<'ast> {
        &self.generic_args
    }

    pub fn is_non_exhaustive(&self) -> bool {
        self.is_non_exhaustive
    }
}
