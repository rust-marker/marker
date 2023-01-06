use crate::{ast::stmt::StmtKind, ffi::FfiSlice};

use super::CommonExprData;

#[repr(C)]
#[derive(Debug)]
pub struct BlockExpr<'ast> {
    data: CommonExprData<'ast>,
    stmts: FfiSlice<'ast, StmtKind<'ast>>,
}

impl<'ast> BlockExpr<'ast> {
    pub fn stmts(&self) -> &[StmtKind<'ast>] {
        self.stmts.get()
    }
}

super::impl_expr_data!(BlockExpr<'ast>, Block);

#[cfg(feature = "driver-api")]
impl<'ast> BlockExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, stmts: &'ast [StmtKind<'ast>]) -> Self {
        Self {
            data,
            stmts: stmts.into(),
        }
    }
}
