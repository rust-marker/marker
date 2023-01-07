use crate::ffi::FfiOption;

use super::{expr::ExprKind, item::ItemKind, pat::PatKind, ty::TyKind};

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum StmtKind<'ast> {
    Item(&'ast ItemKind<'ast>),
    Let(&'ast LetStmt<'ast>),
    Expr(&'ast ExprKind<'ast>),
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LetStmt<'ast> {
    pat: PatKind<'ast>,
    ty: FfiOption<TyKind<'ast>>,
    init_expr: ExprKind<'ast>,
    else_expr: FfiOption<ExprKind<'ast>>,
}

impl<'ast> LetStmt<'ast> {
    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    /// Returns the syntactic type, if it has been specified.
    pub fn ty(&self) -> Option<TyKind<'ast>> {
        self.ty.copy()
    }

    pub fn init_expr(&self) -> ExprKind<'ast> {
        self.init_expr
    }

    pub fn else_expr(&self) -> FfiOption<ExprKind> {
        self.else_expr
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> LetStmt<'ast> {
    pub fn new(
        pat: PatKind<'ast>,
        ty: Option<TyKind<'ast>>,
        init_expr: ExprKind<'ast>,
        else_expr: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            pat,
            ty: ty.into(),
            init_expr,
            else_expr: else_expr.into(),
        }
    }
}
