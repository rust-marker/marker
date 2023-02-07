use marker_api::ast::expr::{
    BlockExpr, BoolLitExpr, CallExpr, CharLitExpr, CommonExprData, ExprKind, ExprPrecedence, FloatLitExpr, FloatSuffix,
    IntLitExpr, IntSuffix, PathExpr, StrLitData, StrLitExpr, UnstableExpr,
};
use rustc_hir as hir;
use std::str::FromStr;

use super::MarkerConversionContext;

impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    #[must_use]
    pub fn to_expr_from_block(&self, block: &hir::Block<'tcx>) -> ExprKind<'ast> {
        let id = self.to_expr_id(block.hir_id);
        if let Some(expr) = self.exprs.borrow().get(&id) {
            return *expr;
        }

        let data = CommonExprData::new(id, self.to_span_id(block.span));
        let expr = ExprKind::Block(self.alloc(|| self.to_block_expr(data, block)));

        self.exprs.borrow_mut().insert(id, expr);
        expr
    }

    #[must_use]
    pub fn to_exprs(&self, exprs: &[hir::Expr<'tcx>]) -> &'ast [ExprKind<'ast>] {
        self.alloc_slice_iter(exprs.iter().map(|expr| self.to_expr(expr)))
    }

    #[must_use]
    pub fn to_expr(&self, expr: &hir::Expr<'tcx>) -> ExprKind<'ast> {
        let id = self.to_expr_id(expr.hir_id);
        if let Some(expr) = self.exprs.borrow().get(&id) {
            return *expr;
        }

        let data = CommonExprData::new(id, self.to_span_id(expr.span));
        let expr =
            match &expr.kind {
                hir::ExprKind::Lit(spanned_lit) => self.to_expr_from_lit_kind(data, &spanned_lit.node),
                hir::ExprKind::Block(block, None) => ExprKind::Block(self.alloc(|| self.to_block_expr(data, block))),
                hir::ExprKind::Call(operand, args) => {
                    ExprKind::Call(self.alloc(|| CallExpr::new(data, self.to_expr(operand), self.to_exprs(args))))
                },
                hir::ExprKind::Path(qpath) => {
                    ExprKind::Path(self.alloc(|| PathExpr::new(data, self.to_qpath_from_expr(qpath, expr))))
                },
                hir::ExprKind::Err => unreachable!("would have triggered a rustc error"),
                _ => {
                    eprintln!("skipping not implemented expr at: {:?}", expr.span);
                    ExprKind::Unstable(self.alloc(|| {
                        UnstableExpr::new(data, ExprPrecedence::Unstable(i32::from(expr.precedence().order())))
                    }))
                },
            };

        self.exprs.borrow_mut().insert(id, expr);
        expr
    }

    #[must_use]
    fn to_block_expr(&self, data: CommonExprData<'ast>, block: &hir::Block<'tcx>) -> BlockExpr<'ast> {
        let stmts: Vec<_> = block.stmts.iter().filter_map(|stmt| self.to_stmt(stmt)).collect();
        let stmts = self.alloc_slice_iter(stmts.into_iter());
        let expr = block.expr.map(|expr| self.to_expr(expr));
        BlockExpr::new(data, stmts, expr)
    }

    fn to_expr_from_lit_kind(&self, data: CommonExprData<'ast>, lit_kind: &rustc_ast::LitKind) -> ExprKind<'ast> {
        match &lit_kind {
            rustc_ast::LitKind::Str(sym, kind) => ExprKind::StrLit(self.alloc(|| {
                StrLitExpr::new(
                    data,
                    matches!(kind, rustc_ast::StrStyle::Raw(_)),
                    StrLitData::Sym(self.to_symbol_id(*sym)),
                )
            })),
            rustc_ast::LitKind::ByteStr(bytes, kind) => ExprKind::StrLit(self.alloc(|| {
                StrLitExpr::new(
                    data,
                    matches!(kind, rustc_ast::StrStyle::Raw(_)),
                    StrLitData::Bytes(self.alloc_slice_iter(bytes.iter().copied()).into()),
                )
            })),
            rustc_ast::LitKind::Byte(value) => {
                ExprKind::IntLit(self.alloc(|| IntLitExpr::new(data, u128::from(*value), None)))
            },
            rustc_ast::LitKind::Char(value) => ExprKind::CharLit(self.alloc(|| CharLitExpr::new(data, *value))),
            rustc_ast::LitKind::Int(value, kind) => {
                let suffix = match kind {
                    rustc_ast::LitIntType::Signed(rustc_ast::IntTy::Isize) => Some(IntSuffix::Isize),
                    rustc_ast::LitIntType::Signed(rustc_ast::IntTy::I8) => Some(IntSuffix::I8),
                    rustc_ast::LitIntType::Signed(rustc_ast::IntTy::I16) => Some(IntSuffix::I16),
                    rustc_ast::LitIntType::Signed(rustc_ast::IntTy::I32) => Some(IntSuffix::I32),
                    rustc_ast::LitIntType::Signed(rustc_ast::IntTy::I64) => Some(IntSuffix::I64),
                    rustc_ast::LitIntType::Signed(rustc_ast::IntTy::I128) => Some(IntSuffix::I128),
                    rustc_ast::LitIntType::Unsigned(rustc_ast::UintTy::Usize) => Some(IntSuffix::Usize),
                    rustc_ast::LitIntType::Unsigned(rustc_ast::UintTy::U8) => Some(IntSuffix::U8),
                    rustc_ast::LitIntType::Unsigned(rustc_ast::UintTy::U16) => Some(IntSuffix::U16),
                    rustc_ast::LitIntType::Unsigned(rustc_ast::UintTy::U32) => Some(IntSuffix::U32),
                    rustc_ast::LitIntType::Unsigned(rustc_ast::UintTy::U64) => Some(IntSuffix::U64),
                    rustc_ast::LitIntType::Unsigned(rustc_ast::UintTy::U128) => Some(IntSuffix::U128),
                    rustc_ast::LitIntType::Unsuffixed => None,
                };
                ExprKind::IntLit(self.alloc(|| IntLitExpr::new(data, *value, suffix)))
            },
            rustc_ast::LitKind::Float(lit_sym, kind) => {
                let suffix = match kind {
                    rustc_ast::LitFloatType::Suffixed(rustc_ast::ast::FloatTy::F32) => Some(FloatSuffix::F32),
                    rustc_ast::LitFloatType::Suffixed(rustc_ast::ast::FloatTy::F64) => Some(FloatSuffix::F64),
                    rustc_ast::LitFloatType::Unsuffixed => None,
                };
                let value = f64::from_str(lit_sym.as_str()).expect("rustc should have validated the literal");
                ExprKind::FloatLit(self.alloc(|| FloatLitExpr::new(data, value, suffix)))
            },
            rustc_ast::LitKind::Bool(value) => ExprKind::BoolLit(self.alloc(|| BoolLitExpr::new(data, *value))),
            rustc_ast::LitKind::Err => unreachable!("would have triggered a rustc error"),
        }
    }
}
