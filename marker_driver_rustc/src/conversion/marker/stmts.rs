use marker_api::ast::stmt::{LetStmt, StmtKind};
use rustc_hir as hir;

use super::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    pub fn to_stmt(&self, stmt: &hir::Stmt<'tcx>) -> Option<StmtKind<'ast>> {
        match &stmt.kind {
            hir::StmtKind::Local(local) => Some(StmtKind::Let(self.alloc(self.to_let_stmt(local)))),
            hir::StmtKind::Item(item) => self.to_item_from_id(*item).map(StmtKind::Item),
            hir::StmtKind::Expr(expr) | hir::StmtKind::Semi(expr) => Some(StmtKind::Expr(self.to_expr(expr))),
        }
    }

    fn to_let_stmt(&self, local: &hir::Local<'tcx>) -> LetStmt<'ast> {
        LetStmt::new(
            self.to_span_id(local.span),
            self.to_pat(local.pat),
            local.ty.map(|ty| self.to_ty(ty)),
            local.init.map(|init| self.to_expr(init)),
            local.els.map(|els| self.to_expr_from_block(els)),
        )
    }
}
