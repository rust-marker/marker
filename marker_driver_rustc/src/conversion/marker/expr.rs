use marker_api::ast::{
    expr::{
        ArrayExpr, BinaryOpExpr, BinaryOpKind, BlockExpr, BoolLitExpr, CallExpr, CharLitExpr, CommonExprData, CtorExpr,
        CtorField, ExprKind, ExprPrecedence, FieldExpr, FloatLitExpr, FloatSuffix, IfExpr, IndexExpr, IntLitExpr,
        IntSuffix, LetExpr, MatchArm, MatchExpr, MethodExpr, PathExpr, RangeExpr, RefExpr, StrLitData, StrLitExpr,
        TupleExpr, UnaryOpExpr, UnaryOpKind, UnstableExpr,
    },
    Ident,
};
use rustc_hir as hir;
use std::str::FromStr;

use super::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_expr_from_block(&self, block: &hir::Block<'tcx>) -> ExprKind<'ast> {
        let id = self.to_expr_id(block.hir_id);
        if let Some(expr) = self.exprs.borrow().get(&id) {
            return *expr;
        }

        let data = CommonExprData::new(id, self.to_span_id(block.span));
        let expr = ExprKind::Block(self.alloc(self.to_block_expr(data, block)));

        self.exprs.borrow_mut().insert(id, expr);
        expr
    }

    #[must_use]
    pub fn to_exprs(&self, exprs: &[hir::Expr<'tcx>]) -> &'ast [ExprKind<'ast>] {
        self.alloc_slice(exprs.iter().map(|expr| self.to_expr(expr)))
    }

    #[must_use]
    pub fn to_expr(&self, expr: &hir::Expr<'tcx>) -> ExprKind<'ast> {
        let id = self.to_expr_id(expr.hir_id);
        if let Some(expr) = self.exprs.borrow().get(&id) {
            return *expr;
        }

        let data = CommonExprData::new(id, self.to_span_id(expr.span));
        let expr = match &expr.kind {
            hir::ExprKind::Lit(spanned_lit) => self.to_expr_from_lit_kind(data, &spanned_lit.node),
            hir::ExprKind::Binary(op, left, right) => ExprKind::BinaryOp(self.alloc(BinaryOpExpr::new(
                data,
                self.to_expr(left),
                self.to_expr(right),
                self.to_bin_op_kind(op),
            ))),
            hir::ExprKind::Unary(op, expr) => {
                ExprKind::UnaryOp(self.alloc(UnaryOpExpr::new(data, self.to_expr(expr), self.to_unary_op_kind(*op))))
            },
            hir::ExprKind::AddrOf(_target, muta, inner) => ExprKind::Ref(self.alloc(RefExpr::new(
                data,
                self.to_expr(inner),
                matches!(muta, hir::Mutability::Mut),
            ))),
            hir::ExprKind::Block(block, None) => ExprKind::Block(self.alloc(self.to_block_expr(data, block))),
            hir::ExprKind::Call(operand, args) => match &operand.kind {
                hir::ExprKind::Path(hir::QPath::LangItem(hir::LangItem::RangeInclusiveNew, _, _)) => {
                    ExprKind::Range(self.alloc({
                        RangeExpr::new(data, Some(self.to_expr(&args[0])), Some(self.to_expr(&args[1])), true)
                    }))
                },
                hir::ExprKind::Path(
                    qpath @ hir::QPath::Resolved(
                        None,
                        hir::Path {
                            // The correct def resolution is done by `to_qpath_from_expr`
                            res: hir::def::Res::Def(hir::def::DefKind::Ctor(_, _), _),
                            ..
                        },
                    ),
                ) => {
                    let fields = self.alloc_slice(args.iter().enumerate().map(|(index, expr)| {
                        CtorField::new(
                            self.to_span_id(expr.span),
                            Ident::new(
                                self.to_symbol_id_for_num(
                                    u32::try_from(index).expect("a index over 2^32 is unexpected"),
                                ),
                                self.to_span_id(rustc_span::DUMMY_SP),
                            ),
                            self.to_expr(expr),
                        )
                    }));
                    ExprKind::Ctor(self.alloc(CtorExpr::new(data, self.to_qpath_from_expr(qpath, expr), fields, None)))
                },

                _ => ExprKind::Call(self.alloc(CallExpr::new(data, self.to_expr(operand), self.to_exprs(args)))),
            },
            hir::ExprKind::MethodCall(method, receiver, args, _span) => ExprKind::Method(self.alloc({
                MethodExpr::new(
                    data,
                    self.to_expr(receiver),
                    self.to_path_segment(method),
                    self.to_exprs(args),
                )
            })),
            hir::ExprKind::Path(
                path @ hir::QPath::Resolved(
                    None,
                    hir::Path {
                        res: hir::def::Res::Def(hir::def::DefKind::Ctor(_, _), ..),
                        ..
                    },
                ),
            ) => ExprKind::Ctor(self.alloc(CtorExpr::new(data, self.to_qpath_from_expr(path, expr), &[], None))),
            hir::ExprKind::Path(qpath) => {
                ExprKind::Path(self.alloc(PathExpr::new(data, self.to_qpath_from_expr(qpath, expr))))
            },
            hir::ExprKind::Tup(exprs) => ExprKind::Tuple(self.alloc(TupleExpr::new(data, self.to_exprs(exprs)))),
            hir::ExprKind::Array(exprs) => {
                ExprKind::Array(self.alloc(ArrayExpr::new(data, self.to_exprs(exprs), None)))
            },
            hir::ExprKind::Repeat(expr, hir::ArrayLen::Body(anon_const)) => {
                let len_body = self.to_body(self.rustc_cx.hir().body(anon_const.body));
                ExprKind::Array(self.alloc(ArrayExpr::new(
                    data,
                    self.alloc_slice([self.to_expr(expr)]),
                    Some(len_body.expr()),
                )))
            },
            hir::ExprKind::Struct(path, fields, base) => match path {
                hir::QPath::LangItem(hir::LangItem::RangeFull, _, _) => {
                    ExprKind::Range(self.alloc(RangeExpr::new(data, None, None, false)))
                },
                hir::QPath::LangItem(hir::LangItem::RangeFrom, _, _) => {
                    ExprKind::Range(self.alloc(RangeExpr::new(data, Some(self.to_expr(fields[0].expr)), None, false)))
                },
                hir::QPath::LangItem(hir::LangItem::RangeTo, _, _) => {
                    ExprKind::Range(self.alloc(RangeExpr::new(data, None, Some(self.to_expr(fields[0].expr)), false)))
                },
                hir::QPath::LangItem(hir::LangItem::Range, _, _) => ExprKind::Range(self.alloc({
                    RangeExpr::new(
                        data,
                        Some(self.to_expr(fields[0].expr)),
                        Some(self.to_expr(fields[1].expr)),
                        false,
                    )
                })),
                hir::QPath::LangItem(hir::LangItem::RangeToInclusive, _, _) => {
                    ExprKind::Range(self.alloc(RangeExpr::new(data, None, Some(self.to_expr(fields[0].expr)), true)))
                },
                _ => {
                    let ctor_fields = self.alloc_slice(fields.iter().map(|field| {
                        CtorField::new(
                            self.to_span_id(field.span),
                            self.to_ident(field.ident),
                            self.to_expr(field.expr),
                        )
                    }));

                    ExprKind::Ctor(self.alloc({
                        CtorExpr::new(
                            data,
                            self.to_qpath_from_expr(path, expr),
                            ctor_fields,
                            base.map(|expr| self.to_expr(expr)),
                        )
                    }))
                },
            },
            hir::ExprKind::Index(operand, index) => {
                ExprKind::Index(self.alloc(IndexExpr::new(data, self.to_expr(operand), self.to_expr(index))))
            },
            hir::ExprKind::Field(operand, field) => {
                ExprKind::Field(self.alloc(FieldExpr::new(data, self.to_expr(operand), self.to_ident(*field))))
            },
            hir::ExprKind::If(scrutinee, then, els) => ExprKind::If(self.alloc(IfExpr::new(
                data,
                self.to_expr(scrutinee),
                self.to_expr(then),
                els.map(|els| self.to_expr(els)),
            ))),
            hir::ExprKind::Let(lets) => self.to_let_expr(lets),
            hir::ExprKind::Match(scrutinee, arms, hir::MatchSource::Normal) => {
                ExprKind::Match(self.alloc(MatchExpr::new(data, self.to_expr(scrutinee), self.to_match_arms(arms))))
            },
            // `DropTemps` is an rustc internal construct to tweak the drop
            // order during HIR lowering. Marker can for now ignore this and
            // convert the inner expression directly
            hir::ExprKind::DropTemps(inner) => self.to_expr(inner),
            hir::ExprKind::Err => unreachable!("would have triggered a rustc error"),
            _ => {
                eprintln!("skipping not implemented expr at: {:?}", expr.span);
                ExprKind::Unstable(
                    self.alloc({
                        UnstableExpr::new(data, ExprPrecedence::Unstable(i32::from(expr.precedence().order())))
                    }),
                )
            },
        };

        self.exprs.borrow_mut().insert(id, expr);
        expr
    }

    #[must_use]
    fn to_block_expr(&self, data: CommonExprData<'ast>, block: &hir::Block<'tcx>) -> BlockExpr<'ast> {
        let stmts: Vec<_> = block.stmts.iter().filter_map(|stmt| self.to_stmt(stmt)).collect();
        let stmts = self.alloc_slice(stmts);
        BlockExpr::new(
            data,
            stmts,
            block.expr.map(|expr| self.to_expr(expr)),
            matches!(block.rules, hir::BlockCheckMode::UnsafeBlock(_)),
        )
    }

    fn to_expr_from_lit_kind(&self, data: CommonExprData<'ast>, lit_kind: &rustc_ast::LitKind) -> ExprKind<'ast> {
        match &lit_kind {
            rustc_ast::LitKind::Str(sym, kind) => ExprKind::StrLit(self.alloc({
                StrLitExpr::new(
                    data,
                    matches!(kind, rustc_ast::StrStyle::Raw(_)),
                    StrLitData::Sym(self.to_symbol_id(*sym)),
                )
            })),
            rustc_ast::LitKind::ByteStr(bytes, kind) => ExprKind::StrLit(self.alloc({
                StrLitExpr::new(
                    data,
                    matches!(kind, rustc_ast::StrStyle::Raw(_)),
                    StrLitData::Bytes(self.alloc_slice(bytes.iter().copied()).into()),
                )
            })),
            rustc_ast::LitKind::Byte(value) => {
                ExprKind::IntLit(self.alloc(IntLitExpr::new(data, u128::from(*value), None)))
            },
            rustc_ast::LitKind::Char(value) => ExprKind::CharLit(self.alloc(CharLitExpr::new(data, *value))),
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
                ExprKind::IntLit(self.alloc(IntLitExpr::new(data, *value, suffix)))
            },
            rustc_ast::LitKind::Float(lit_sym, kind) => {
                let suffix = match kind {
                    rustc_ast::LitFloatType::Suffixed(rustc_ast::ast::FloatTy::F32) => Some(FloatSuffix::F32),
                    rustc_ast::LitFloatType::Suffixed(rustc_ast::ast::FloatTy::F64) => Some(FloatSuffix::F64),
                    rustc_ast::LitFloatType::Unsuffixed => None,
                };
                let value = f64::from_str(lit_sym.as_str()).expect("rustc should have validated the literal");
                ExprKind::FloatLit(self.alloc(FloatLitExpr::new(data, value, suffix)))
            },
            rustc_ast::LitKind::Bool(value) => ExprKind::BoolLit(self.alloc(BoolLitExpr::new(data, *value))),
            rustc_ast::LitKind::Err => unreachable!("would have triggered a rustc error"),
        }
    }

    fn to_bin_op_kind(&self, op: &hir::BinOp) -> BinaryOpKind {
        match op.node {
            hir::BinOpKind::Add => BinaryOpKind::Add,
            hir::BinOpKind::Sub => BinaryOpKind::Sub,
            hir::BinOpKind::Mul => BinaryOpKind::Mul,
            hir::BinOpKind::Div => BinaryOpKind::Div,
            hir::BinOpKind::Rem => BinaryOpKind::Rem,
            hir::BinOpKind::And => BinaryOpKind::And,
            hir::BinOpKind::Or => BinaryOpKind::Or,
            hir::BinOpKind::BitXor => BinaryOpKind::BitXor,
            hir::BinOpKind::BitAnd => BinaryOpKind::BitAnd,
            hir::BinOpKind::BitOr => BinaryOpKind::BitOr,
            hir::BinOpKind::Shl => BinaryOpKind::Shl,
            hir::BinOpKind::Shr => BinaryOpKind::Shr,
            hir::BinOpKind::Eq => BinaryOpKind::Eq,
            hir::BinOpKind::Lt => BinaryOpKind::Lesser,
            hir::BinOpKind::Le => BinaryOpKind::LesserEq,
            hir::BinOpKind::Ne => BinaryOpKind::NotEq,
            hir::BinOpKind::Ge => BinaryOpKind::GreaterEq,
            hir::BinOpKind::Gt => BinaryOpKind::Greater,
        }
    }

    fn to_unary_op_kind(&self, op: hir::UnOp) -> UnaryOpKind {
        match op {
            hir::UnOp::Neg => UnaryOpKind::Neg,
            hir::UnOp::Not => UnaryOpKind::Not,
            hir::UnOp::Deref => UnaryOpKind::Deref,
        }
    }

    fn to_match_arms(&self, arms: &[hir::Arm<'tcx>]) -> &'ast [MatchArm<'ast>] {
        self.alloc_slice(arms.iter().map(|arm| self.to_match_arm(arm)))
    }

    fn to_match_arm(&self, arm: &hir::Arm<'tcx>) -> MatchArm<'ast> {
        let guard = match &arm.guard {
            Some(hir::Guard::If(expr)) => Some(self.to_expr(expr)),
            Some(hir::Guard::IfLet(lets)) => Some(self.to_let_expr(lets)),
            None => None,
        };
        MatchArm::new(
            self.to_span_id(arm.span),
            self.to_pat(arm.pat),
            guard,
            self.to_expr(arm.body),
        )
    }

    fn to_let_expr(&self, lets: &hir::Let<'tcx>) -> ExprKind<'ast> {
        let data = CommonExprData::new(self.to_expr_id(lets.hir_id), self.to_span_id(lets.span));
        ExprKind::Let(self.alloc(LetExpr::new(data, self.to_pat(lets.pat), self.to_expr(lets.init))))
    }
}
