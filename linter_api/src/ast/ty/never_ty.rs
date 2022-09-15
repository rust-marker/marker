use super::CommonTyData;

#[repr(C)]
pub struct NeverTy<'ast> {
    data: CommonTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> NeverTy<'ast> {
    pub fn new(data: CommonTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(NeverTy<'ast>, Never);

impl<'ast> std::fmt::Debug for NeverTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("!").finish()
    }
}
