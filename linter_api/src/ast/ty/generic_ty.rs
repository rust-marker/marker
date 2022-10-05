use crate::ast::AstPath;

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GenericTy<'ast> {
    data: CommonTyData<'ast>,
    path: &'ast AstPath<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> GenericTy<'ast> {}

super::impl_ty_data!(GenericTy<'ast>, Generic);
