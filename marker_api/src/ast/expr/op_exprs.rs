use crate::{
    ast::{pat::PatKind, ty::TyKind},
    ffi::FfiOption,
};

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
pub struct RefExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
    is_mut: bool,
}

impl<'ast> RefExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }
}

super::impl_expr_data!(
    RefExpr<'ast>,
    Ref,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Ref
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> RefExpr<'ast> {
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

/// An expression assigning a value to an assignee expression.
///
/// ```
///     let mut assignee = 20;
///
/// //  vvvvvvvv The assignee expression
///     assignee = 10;
/// //             ^^ The value expression
///
/// //  vvvvvvvvvvvvv A complex assignee expression
///     [assignee, _] = [2, 3];
/// //                ^ ^^^^^^ The value expression
/// //                |
/// //                No compound operator
///
/// //  vvvvvvvv The assignee expression
///     assignee += 1;
/// //           ^  ^ The value expression
/// //           |
/// //           Plus as a compound operator
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct AssignExpr<'ast> {
    data: CommonExprData<'ast>,
    assignee: PatKind<'ast>,
    value: ExprKind<'ast>,
    op: FfiOption<BinaryOpKind>,
}

impl<'ast> AssignExpr<'ast> {
    pub fn assignee(&self) -> PatKind<'ast> {
        self.assignee
    }

    pub fn value(&self) -> ExprKind<'ast> {
        self.value
    }

    pub fn op(&self) -> Option<BinaryOpKind> {
        self.op.copy()
    }
}

super::impl_expr_data!(AssignExpr<'ast>, Assign);

#[cfg(feature = "driver-api")]
impl<'ast> AssignExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        assignee: PatKind<'ast>,
        value: ExprKind<'ast>,
        op: Option<BinaryOpKind>,
    ) -> Self {
        Self {
            data,
            assignee,
            value,
            op: op.into(),
        }
    }
}
