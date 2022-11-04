use crate::{
    ast::{generic::GenericArgs, DefTyId},
    ffi::FfiSlice,
};

use super::{CommonTyData, FieldDef};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct UnionTy<'ast> {
    data: CommonTyData<'ast>,
    def_id: DefTyId,
    generic_args: GenericArgs<'ast>,
    fields: FfiSlice<'ast, &'ast FieldDef<'ast>>,
    // FIXME: Add representation/layout info like alignment, size, type
}

#[cfg(feature = "driver-api")]
impl<'ast> UnionTy<'ast> {
    pub fn new(
        data: CommonTyData<'ast>,
        def_id: DefTyId,
        generic_args: GenericArgs<'ast>,
        fields: &'ast [&'ast FieldDef<'ast>],
    ) -> Self {
        Self {
            data,
            def_id,
            generic_args,
            fields: fields.into(),
        }
    }
}

super::impl_ty_data!(UnionTy<'ast>, Union);

impl<'ast> UnionTy<'ast> {
    pub fn def_id(&self) -> DefTyId {
        self.def_id
    }

    pub fn generic_args(&self) -> &GenericArgs<'ast> {
        &self.generic_args
    }

    pub fn fields(&self) -> &[&FieldDef<'ast>] {
        self.fields.get()
    }
}
