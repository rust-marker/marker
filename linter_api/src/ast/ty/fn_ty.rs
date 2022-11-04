use crate::ast::{impl_callable_trait, CallableData};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FnTy<'ast> {
    data: CommonTyData<'ast>,
    callable_data: CallableData<'ast>,
    // FIXME: Add source information and methods to check what this constructs
    // in cases were this is a reference to a constructor
}

#[cfg(feature = "driver-api")]
impl<'ast> FnTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, callable_data: CallableData<'ast>) -> Self {
        Self { data, callable_data }
    }
}

super::impl_ty_data!(FnTy<'ast>, Fn);
impl_callable_trait!(FnTy<'ast>);
