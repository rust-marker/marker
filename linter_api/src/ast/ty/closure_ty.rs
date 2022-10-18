use crate::ast::{impl_callable_trait, CallableData};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClosureTy<'ast> {
    data: CommonTyData<'ast>,
    callable_data: CallableData<'ast>,
    // FIXME: Add support for `for<'lifetime>` binder
    // FIXME: Potentualy add functions to check which `Fn` traits this implements
}

#[cfg(feature = "driver-api")]
impl<'ast> ClosureTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, callable_data: CallableData<'ast>) -> Self {
        Self { data, callable_data }
    }
}

super::impl_ty_data!(ClosureTy<'ast>, Closure);
impl_callable_trait!(ClosureTy<'ast>);
