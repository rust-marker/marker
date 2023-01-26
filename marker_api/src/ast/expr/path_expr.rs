use crate::ast::QualifiedAstPath;

use super::CommonExprData;

#[repr(C)]
#[derive(Debug)]
pub struct PathExpr<'ast> {
    data: CommonExprData<'ast>,
    path: QualifiedAstPath<'ast>,
}

impl<'ast> PathExpr<'ast> {
    /// Returns the [`QualifiedAstPath`] as defined in the expression.
    /// [`QualifiedAstPath::resolve()`] can be used to resolve the node
    /// referenced by this path.
    pub fn path(&self) -> &QualifiedAstPath<'ast> {
        &self.path
    }
}

super::impl_expr_data!(PathExpr<'ast>, Path);

#[cfg(feature = "driver-api")]
impl<'ast> PathExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, path: QualifiedAstPath<'ast>) -> Self {
        Self { data, path }
    }
}
