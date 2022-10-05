use super::CommonTyData;

#[repr(C)]
#[derive(Debug)]
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
