use std::marker::PhantomData;

use super::CommonSynTyData;

/// The placeholder type, signalling that the semantic type is still unstable
/// and therefor not represented as part of the API.
#[repr(C)]
#[derive(Debug)]
pub struct SemUnstableTy<'ast> {
    _lt: PhantomData<&'ast ()>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SemUnstableTy<'ast> {
    pub fn new() -> Self {
        Self { _lt: PhantomData }
    }
}

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
