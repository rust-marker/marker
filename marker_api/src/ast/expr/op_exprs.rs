use crate::{
    ast::{pat::PatKind, ty::TyKind},
    common::Mutability,
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
    mutability: Mutability,
}

impl<'ast> RefExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    pub fn mutability(&self) -> Mutability {
        self.mutability
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
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>, mutability: Mutability) -> Self {
        Self { data, expr, mutability }
    }
}

/// The `?` operator that unwraps valid values or propagates erroneous values to
/// the the calling function.
///
/// Here is an example of the operator:
///
/// ```
/// fn try_option_example(opt: Option<i32>) -> Option<String> {
///     // The `?` operator unwraps the value if `opt` is `Some` or
///     // propagates `None` if `opt` is empty.
///     //             v
///     let value = opt?;
///     // `value` has the type `i32`
///     
///     // [...]
///     # Some(value.to_string())
/// }
///
/// fn try_result_example(res: Result<i32, ()>) -> Result<String, ()> {
///     // The `?` operator unwraps the value if `res` is `Ok` or
///     // propagates the value of `Err` if `res` is an error.
///     //             v
///     let value = res?;
///     // `value` has the type `i32`
///     
///     // [...]
///     # Ok(value.to_string())
/// }
/// ```
///
/// This operator is also known as the *question mark* or *error propagation* operator.
///
/// See <https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator>
#[repr(C)]
#[derive(Debug)]
pub struct TryExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
}

impl<'ast> TryExpr<'ast> {
    /// The expression that might produce an error, that would be propagated by
    /// this operator.
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

super::impl_expr_data!(TryExpr<'ast>, Try);

#[cfg(feature = "driver-api")]
impl<'ast> TryExpr<'ast> {
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    cast_ty: TyKind<'ast>,
}

impl<'ast> AsExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    pub fn cast_ty(&self) -> TyKind<'ast> {
        self.cast_ty
    }
}

super::impl_expr_data!(AsExpr<'ast>, As);

#[cfg(feature = "driver-api")]
impl<'ast> AsExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>, cast_ty: TyKind<'ast>) -> Self {
        Self { data, expr, cast_ty }
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

/// An `.await` expression on a future, like:
///
/// ```
/// # async fn foo() -> u8 {
/// #     16
/// # }
/// # async fn wrapper() {
/// // The await expression
/// //  vvvvvvvvvvv
///     foo().await;
/// //  ^^^^^
/// // The future, that will be awaited
/// # }
/// ```
///
/// Marker specificity hides the desugar of `.await` expressions. The [Rust Reference]
/// contains more information how rustc desugars `.await` expressions.
///
/// [Rust Reference]: <https://doc.rust-lang.org/reference/expressions/await-expr.html>
#[repr(C)]
#[derive(Debug)]
pub struct AwaitExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: ExprKind<'ast>,
}

impl<'ast> AwaitExpr<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

super::impl_expr_data!(AwaitExpr<'ast>, Await);

#[cfg(feature = "driver-api")]
impl<'ast> AwaitExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: ExprKind<'ast>) -> Self {
        Self { data, expr }
    }
}
