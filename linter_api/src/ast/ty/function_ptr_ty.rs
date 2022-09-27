use crate::ast::{impl_callable_trait, CallableData};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug)]
pub struct FunctionPtrTy<'ast> {
    data: CommonTyData<'ast>,
    callable_data: CallableData<'ast>,
    // FIXME: Add `for<'a>` bound
}

#[cfg(feature = "driver-api")]
impl<'ast> FunctionPtrTy<'ast> {}

super::impl_ty_data!(FunctionPtrTy<'ast>, FunctionPtr);
impl_callable_trait!(FunctionPtrTy<'ast>);

impl<'ast> FunctionPtrTy<'ast> {}
