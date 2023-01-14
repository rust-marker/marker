use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct RefPat<'ast> {
    data: CommonPatData<'ast>,
    pattern: PatKind<'ast>,
    is_mut: bool,
}

impl<'ast> RefPat<'ast> {
    /// Returns the pattern, that is behind this reference pattern.
    pub fn pattern(&self) -> PatKind<'ast> {
        self.pattern
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }
}

super::impl_pat_data!(RefPat<'ast>, Ref);

#[cfg(feature = "driver-api")]
impl<'ast> RefPat<'ast> {
    pub fn new(data: CommonPatData<'ast>, pattern: PatKind<'ast>, is_mut: bool) -> Self {
        Self { data, pattern, is_mut }
    }
}
