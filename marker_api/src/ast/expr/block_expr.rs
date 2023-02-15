use crate::{
    ast::stmt::StmtKind,
    ffi::{FfiOption, FfiSlice},
};

use super::{CommonExprData, ExprKind};

#[repr(C)]
#[derive(Debug)]
pub struct BlockExpr<'ast> {
    data: CommonExprData<'ast>,
    stmts: FfiSlice<'ast, StmtKind<'ast>>,
    expr: FfiOption<ExprKind<'ast>>,
    is_unsafe: bool,
}

impl<'ast> BlockExpr<'ast> {
    /// This returns all statements of this block. The optional value expression,
    /// which is returned by the block, is stored separately. See [`BlockExpr::expr()`]
    pub fn stmts(&self) -> &[StmtKind<'ast>] {
        self.stmts.get()
    }

    /// Blocks may optionally end with an expression, indicated by an expression
    /// without a trailing semicolon.
    pub fn expr(&self) -> Option<ExprKind<'ast>> {
        self.expr.copy()
    }

    pub fn is_unsafe(&self) -> bool {
        self.is_unsafe
    }
}

super::impl_expr_data!(BlockExpr<'ast>, Block);

#[cfg(feature = "driver-api")]
impl<'ast> BlockExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        stmts: &'ast [StmtKind<'ast>],
        expr: Option<ExprKind<'ast>>,
        is_unsafe: bool,
    ) -> Self {
        Self {
            data,
            stmts: stmts.into(),
            expr: expr.into(),
            is_unsafe,
        }
    }
}
