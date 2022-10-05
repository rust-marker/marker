use super::CommonTyData;

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct BooleanTy<'ast> {
    data: CommonTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> BooleanTy<'ast> {
    pub fn new(data: CommonTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(BooleanTy<'ast>, Boolean);

impl<'ast> std::fmt::Debug for BooleanTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}
