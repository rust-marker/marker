use crate::ffi::FfiSlice;

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct SlicePat<'ast> {
    data: CommonPatData<'ast>,
    elements: FfiSlice<'ast, PatKind<'ast>>,
}

impl<'ast> SlicePat<'ast> {
    pub fn elements(&self) -> &[PatKind<'ast>] {
        (&self.elements).into()
    }
}

super::impl_pat_data!(SlicePat<'ast>, Slice);

#[cfg(feature = "driver-api")]
impl<'ast> SlicePat<'ast> {
    pub fn new(data: CommonPatData<'ast>, elements: &'ast [PatKind<'ast>]) -> Self {
        Self {
            data,
            elements: elements.into(),
        }
    }
}
