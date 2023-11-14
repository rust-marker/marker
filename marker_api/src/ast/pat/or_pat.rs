use crate::ffi::FfiSlice;

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct OrPat<'ast> {
    data: CommonPatData<'ast>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    pats: FfiSlice<'ast, PatKind<'ast>>,
}

impl<'ast> OrPat<'ast> {
    /// The patterns, which are considered by this or pattern.
    pub fn pats(&self) -> &[PatKind<'ast>] {
        (&self.pats).into()
    }
}

super::impl_pat_data!(OrPat<'ast>, Or);

#[cfg(feature = "driver-api")]
impl<'ast> OrPat<'ast> {
    pub fn new(data: CommonPatData<'ast>, patterns: &'ast [PatKind<'ast>]) -> Self {
        Self {
            data,
            pats: patterns.into(),
        }
    }
}
