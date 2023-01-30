use crate::ast::AstQPath;

use super::CommonExprData;

#[repr(C)]
#[derive(Debug)]
pub struct PathExpr<'ast> {
    data: CommonExprData<'ast>,
    path: AstQPath<'ast>,
}

impl<'ast> PathExpr<'ast> {
    /// Returns the [`AstQPath`] as defined in the expression.
    /// [`AstQPath::resolve()`] can be used to resolve the node referenced
    /// by this path.
    pub fn path(&self) -> &AstQPath<'ast> {
        &self.path
    }
}

super::impl_expr_data!(PathExpr<'ast>, Path);

#[cfg(feature = "driver-api")]
impl<'ast> PathExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, path: AstQPath<'ast>) -> Self {
        Self { data, path }
    }
}
