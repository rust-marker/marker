use crate::common::Mutability;

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct RefPat<'ast> {
    data: CommonPatData<'ast>,
    pat: PatKind<'ast>,
    mutability: Mutability,
}

impl<'ast> RefPat<'ast> {
    /// Returns the pattern, that is behind this reference pattern.
    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    pub fn mutability(&self) -> Mutability {
        self.mutability
    }
}

super::impl_pat_data!(RefPat<'ast>, Ref);

#[cfg(feature = "driver-api")]
impl<'ast> RefPat<'ast> {
    pub fn new(data: CommonPatData<'ast>, pat: PatKind<'ast>, mutability: Mutability) -> Self {
        Self { data, pat, mutability }
    }
}
