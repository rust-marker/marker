use crate::ast::GenericId;

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GenericTy<'ast> {
    data: CommonTyData<'ast>,
    generic_id: GenericId,
}

#[cfg(feature = "driver-api")]
impl<'ast> GenericTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, generic_id: GenericId) -> Self {
        Self { data, generic_id }
    }
}

super::impl_ty_data!(GenericTy<'ast>, Generic);
