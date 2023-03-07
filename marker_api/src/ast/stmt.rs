use crate::{context::with_cx, ffi::FfiOption};

use super::{expr::ExprKind, item::ItemKind, pat::PatKind, ty::TyKind, LetStmtId, Span, SpanId, StmtId, StmtIdInner};

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum StmtKind<'ast> {
    Item(ItemKind<'ast>),
    Let(&'ast LetStmt<'ast>),
    Expr(ExprKind<'ast>),
}

impl<'ast> StmtKind<'ast> {
    pub fn id(&self) -> StmtId {
        match self {
            StmtKind::Item(node) => StmtId::ast_new(StmtIdInner::Item(node.id())),
            StmtKind::Let(node) => node.id(),
            StmtKind::Expr(node) => StmtId::ast_new(StmtIdInner::Expr(node.id())),
        }
    }

    pub fn span(&self) -> &Span<'ast> {
        match self {
            StmtKind::Item(node) => node.span(),
            StmtKind::Let(node) => node.span(),
            StmtKind::Expr(node) => node.span(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LetStmt<'ast> {
    id: LetStmtId,
    span: SpanId,
    pat: PatKind<'ast>,
    ty: FfiOption<TyKind<'ast>>,
    init: FfiOption<ExprKind<'ast>>,
    els: FfiOption<ExprKind<'ast>>,
}

impl<'ast> LetStmt<'ast> {
    pub fn id(&self) -> StmtId {
        StmtId::ast_new(StmtIdInner::LetStmt(self.id))
    }

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

    pub fn init(&self) -> Option<ExprKind<'ast>> {
        self.init.copy()
    }

    /// This returns the optional `else` expression of the let statement.
    ///
    /// `els` is an abbreviation for `else`, which is a reserved keyword in Rust.
    pub fn els(&self) -> Option<ExprKind> {
        self.els.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> LetStmt<'ast> {
    pub fn new(
        id: LetStmtId,
        span: SpanId,
        pat: PatKind<'ast>,
        ty: Option<TyKind<'ast>>,
        init: Option<ExprKind<'ast>>,
        els: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            id,
            span,
            pat,
            ty: ty.into(),
            init: init.into(),
            els: els.into(),
        }
    }
}
