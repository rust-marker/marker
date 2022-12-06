use crate::ffi::FfiSlice;

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct TuplePat<'ast> {
    data: CommonPatData<'ast>,
    elements: FfiSlice<'ast, PatKind<'ast>>,
}

impl<'ast> TuplePat<'ast> {
    pub fn elements(&self) -> &[PatKind<'ast>] {
        (&self.elements).into()
    }
}

super::impl_pat_data!(TuplePat<'ast>, Tuple);

#[cfg(feature = "driver-api")]
impl<'ast> TuplePat<'ast> {
    pub fn new(data: CommonPatData<'ast>, elements: &'ast [PatKind<'ast>]) -> Self {
        Self {
            data,
            elements: elements.into(),
        }
    }
}
