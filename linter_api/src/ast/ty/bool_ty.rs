use super::CommonTyData;

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct BoolTy<'ast> {
    data: CommonTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> BoolTy<'ast> {
    pub fn new(data: CommonTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(BoolTy<'ast>, Bool);

impl<'ast> std::fmt::Debug for BoolTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}
