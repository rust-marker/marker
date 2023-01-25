use crate::ast::ty::TyKind;

use super::{CommonExprData, ExprKind, ExprPrecedence};

#[repr(C)]
#[derive(Debug)]
pub struct BinaryOpExpr<'ast> {
    data: CommonExprData<'ast>,
    left: ExprKind<'ast>,
    right: ExprKind<'ast>,
    kind: BinaryOpKind,
}

impl<'ast> BinaryOpExpr<'ast> {
    pub fn left(&self) -> ExprKind<'ast> {
        self.left
    }

    pub fn right(&self) -> ExprKind<'ast> {
        self.right
    }

    pub fn kind(&self) -> BinaryOpKind {
        self.kind
    }
}

super::impl_expr_data!(
    BinaryOpExpr<'ast>,
    BinaryOp,
    fn precedence(&self) -> ExprPrecedence {
        match self.kind {
            BinaryOpKind::Mul => ExprPrecedence::Mul,
            BinaryOpKind::Div => ExprPrecedence::Div,
            BinaryOpKind::Rem => ExprPrecedence::Rem,
            BinaryOpKind::Add => ExprPrecedence::Add,
            BinaryOpKind::Sub => ExprPrecedence::Sub,
            BinaryOpKind::Shr => ExprPrecedence::Shr,
            BinaryOpKind::Shl => ExprPrecedence::Shl,
            BinaryOpKind::BitAnd => ExprPrecedence::BitAnd,
            BinaryOpKind::BitXor => ExprPrecedence::BitXor,
            BinaryOpKind::BitOr => ExprPrecedence::BitOr,
            BinaryOpKind::Eq
            | BinaryOpKind::Greater
            | BinaryOpKind::GreaterEq
            | BinaryOpKind::Lesser
            | BinaryOpKind::LesserEq
            | BinaryOpKind::NotEq => ExprPrecedence::Comparison,
            BinaryOpKind::And => ExprPrecedence::And,
            BinaryOpKind::Or => ExprPrecedence::Or,
        }
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> BinaryOpExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, left: ExprKind<'ast>, right: ExprKind<'ast>, kind: BinaryOpKind) -> Self {
        Self {
            data,
            left,
            right,
            kind,
        }
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum BinaryOpKind {
    /// The `*` operator
    Mul,
    /// The `/` operator
    Div,
    /// The `%` operator
    Rem,
    /// The `+` operator
    Add,
    /// The `-` operator
    Sub,
    /// The `>>` operator
    Shr,
    /// The `<<` operator
    Shl,
    /// The `&` operator
    BitAnd,
    /// The `^` operator
    BitXor,
    /// The `|` operator
    BitOr,
    /// The `==` operator
    Eq,
    /// The `!=` operator
    NotEq,
    /// The `>` operator
    Greater,
    /// The `>=` operator
    GreaterEq,
    /// The `<` operator
    Lesser,
    /// The `<=` operator
    LesserEq,
    /// The `&&` operator
    And,
    /// The `||` operator
    Or,
}

#[repr(C)]
#[derive(Debug)]
// FIXME, `ReferenceExpr` might be a better name for this. Thoughts?
pub struct BorrowExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
    is_mut: bool,
}

impl<'ast> BorrowExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }
}

super::impl_expr_data!(
    BorrowExpr<'ast>,
    Borrow,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Reference
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> BorrowExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>, is_mut: bool) -> Self {
        Self { data, expr, is_mut }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct QuestionMarkExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
}

impl<'ast> QuestionMarkExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

super::impl_expr_data!(QuestionMarkExpr<'ast>, QuestionMark);

#[cfg(feature = "driver-api")]
impl<'ast> QuestionMarkExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>) -> Self {
        Self { data, expr }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct UnaryOpExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
    kind: UnaryOpKind,
}

impl<'ast> UnaryOpExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    pub fn kind(&self) -> UnaryOpKind {
        self.kind
    }
}

super::impl_expr_data!(
    UnaryOpExpr<'ast>,
    UnaryOp,
    fn precedence(&self) -> ExprPrecedence {
        match self.kind {
            UnaryOpKind::Neg => ExprPrecedence::Neg,
            UnaryOpKind::Not => ExprPrecedence::Not,
            UnaryOpKind::Deref => ExprPrecedence::Deref,
        }
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> UnaryOpExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>, kind: UnaryOpKind) -> Self {
        Self { data, expr, kind }
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum UnaryOpKind {
    /// The arithmetic negation `-` operator, like `-2`
    Neg,
    /// The logical negation `!` operator, like `!true`
    Not,
    /// The dereference `*` operator, like `*value`
    Deref,
}

#[repr(C)]
#[derive(Debug)]
pub struct AsExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
    ty: TyKind<'ast>,
}

impl<'ast> AsExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }
}

super::impl_expr_data!(AsExpr<'ast>, As);

#[cfg(feature = "driver-api")]
impl<'ast> AsExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>, ty: TyKind<'ast>) -> Self {
        Self { data, expr, ty }
    }
}

// FIXME: Add Assign expressions, these will require place expressions and a decision
// if some cases should be represented as patterns or always as expressions.
