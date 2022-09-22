use super::CommonTyData;

#[repr(C)]
#[derive(Debug)]
pub struct FunctionPtrTy<'ast> {
    data: CommonTyData<'ast>,
    // FIXME: Add `for<'a>` bound
}

#[cfg(feature = "driver-api")]
impl<'ast> FunctionPtrTy<'ast> {}

super::impl_ty_data!(FunctionPtrTy<'ast>, FunctionPtr);

impl<'ast> FunctionPtrTy<'ast> {}
