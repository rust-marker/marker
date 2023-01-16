use crate::{context::with_cx, ffi::FfiOption};

use super::{expr::ExprKind, item::ItemKind, pat::PatKind, ty::TyKind, Span, SpanId};

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
    span: SpanId,
    pat: PatKind<'ast>,
    ty: FfiOption<TyKind<'ast>>,
    init_expr: FfiOption<ExprKind<'ast>>,
    else_expr: FfiOption<ExprKind<'ast>>,
}

impl<'ast> LetStmt<'ast> {
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }

    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    /// Returns the syntactic type, if it has been specified.
    pub fn ty(&self) -> Option<TyKind<'ast>> {
        self.ty.copy()
    }

    pub fn init_expr(&self) -> Option<ExprKind<'ast>> {
        self.init_expr.copy()
    }

    pub fn else_expr(&self) -> Option<ExprKind> {
        self.else_expr.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> LetStmt<'ast> {
    pub fn new(
        span: SpanId,
        pat: PatKind<'ast>,
        ty: Option<TyKind<'ast>>,
        init_expr: Option<ExprKind<'ast>>,
        else_expr: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            span,
            pat,
            ty: ty.into(),
            init_expr: init_expr.into(),
            else_expr: else_expr.into(),
        }
    }
}
