use super::CommonTyData;

#[repr(C)]
pub struct InferredTy<'ast> {
    data: CommonTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> InferredTy<'ast> {
    pub fn new(data: CommonTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(InferredTy<'ast>, Inferred);

impl<'ast> std::fmt::Debug for InferredTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("!").finish()
    }
}
