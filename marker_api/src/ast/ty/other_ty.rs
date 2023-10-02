use super::CommonSynTyData;

#[repr(C)]
#[derive(Debug)]
pub struct SynInferredTy<'ast> {
    data: CommonSynTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SynInferredTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(SynInferredTy<'ast>, Inferred);
