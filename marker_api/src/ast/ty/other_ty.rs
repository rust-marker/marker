use super::CommonSynTyData;

#[repr(C)]
#[derive(Debug)]
pub struct InferredTy<'ast> {
    data: CommonSynTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> InferredTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(InferredTy<'ast>, Inferred);
