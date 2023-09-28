use crate::ast::AstQPath;

use super::CommonPatData;

#[repr(C)]
#[derive(Debug)]
pub struct PathPat<'ast> {
    data: CommonPatData<'ast>,
    path: AstQPath<'ast>,
}

impl<'ast> PathPat<'ast> {
    /// Returns the [`AstQPath`] as defined in the expression.
    /// [`AstQPath::resolve()`] can be used to resolve the node referenced
    /// by this path.
    pub fn path(&self) -> &AstQPath<'ast> {
        &self.path
    }
}

super::impl_pat_data!(PathPat<'ast>, Path);

#[cfg(feature = "driver-api")]
impl<'ast> PathPat<'ast> {
    pub fn new(data: CommonPatData<'ast>, path: AstQPath<'ast>) -> Self {
        Self { data, path }
    }
}
