use crate::ffi::FfiSlice;

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct OrPat<'ast> {
    data: CommonPatData<'ast>,
    patterns: FfiSlice<'ast, PatKind<'ast>>,
}

impl<'ast> OrPat<'ast> {
    pub fn patterns(&self) -> &[PatKind<'ast>] {
        (&self.patterns).into()
    }
}

super::impl_pat_data!(OrPat<'ast>, Or);

#[cfg(feature = "driver-api")]
impl<'ast> OrPat<'ast> {
    pub fn new(data: CommonPatData<'ast>, patterns: &'ast [PatKind<'ast>]) -> Self {
        Self {
            data,
            patterns: patterns.into(),
        }
    }
}
