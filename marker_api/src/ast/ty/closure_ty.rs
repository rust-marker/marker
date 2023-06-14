use crate::ast::{impl_callable_data_trait, CommonCallableData};

use super::CommonSynTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClosureTy<'ast> {
    data: CommonSynTyData<'ast>,
    callable_data: CommonCallableData<'ast>,
    // FIXME: Add support for `for<'lifetime>` binder
    // FIXME: Potentially add functions to check which [`Fn`] traits this implements
}

#[cfg(feature = "driver-api")]
impl<'ast> ClosureTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, callable_data: CommonCallableData<'ast>) -> Self {
        Self { data, callable_data }
    }
}

super::impl_ty_data!(ClosureTy<'ast>, Closure);
impl_callable_data_trait!(ClosureTy<'ast>);
