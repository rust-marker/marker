use crate::ast::{impl_callable_trait, CallableData};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FnPtrTy<'ast> {
    data: CommonTyData<'ast>,
    callable_data: CallableData<'ast>,
    // FIXME: Add `for<'a>` bound
}

#[cfg(feature = "driver-api")]
impl<'ast> FnPtrTy<'ast> {}

super::impl_ty_data!(FnPtrTy<'ast>, FnPtr);
impl_callable_trait!(FnPtrTy<'ast>);

impl<'ast> FnPtrTy<'ast> {}
