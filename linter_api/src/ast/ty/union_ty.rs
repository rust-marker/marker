use crate::{
    ast::{item::Generics, DefTyId},
    ffi::FfiSlice,
};

use super::{CommonTyData, FieldDef};

#[repr(C)]
#[derive(Debug)]
pub struct UnionTy<'ast> {
    data: CommonTyData<'ast>,
    def_id: DefTyId,
    generics: Generics<'ast>,
    fields: FfiSlice<'ast, &'ast FieldDef<'ast>>,
    // FIXME: Add representation/layout info like alignment, size, type
}

#[cfg(feature = "driver-api")]
impl<'ast> UnionTy<'ast> {
    pub fn new(
        data: CommonTyData<'ast>,
        def_id: DefTyId,
        generics: Generics<'ast>,
        fields: FfiSlice<'ast, &'ast FieldDef<'ast>>,
    ) -> Self {
        Self {
            data,
            def_id,
            generics,
            fields,
        }
    }
}

super::impl_ty_data!(UnionTy<'ast>, Union);

impl<'ast> UnionTy<'ast> {
    pub fn def_id(&self) -> DefTyId {
        self.def_id
    }

    pub fn generics(&self) -> &Generics<'ast> {
        &self.generics
    }

    pub fn fields(&self) -> &[&FieldDef<'ast>] {
        self.fields.get()
    }
}
