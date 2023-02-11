use super::{ty::TyKind, ExprId, Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

mod block_expr;
mod call_exprs;
mod ctor_expr;
mod lit_expr;
mod op_exprs;
mod path_expr;
mod unstable_expr;
pub use block_expr::*;
pub use call_exprs::*;
pub use ctor_expr::*;
pub use lit_expr::*;
pub use op_exprs::*;
pub use path_expr::*;
pub use unstable_expr::*;

pub trait ExprData<'ast>: Debug {
    fn id(&self) -> ExprId;

    /// Returns the span of this expression.
    fn span(&self) -> &Span<'ast>;

    // This returns the semantic type of this expression
    fn ty(&self) -> TyKind<'ast>;

    fn precedence(&self) -> ExprPrecedence;

    fn as_expr(&'ast self) -> ExprKind<'ast>;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprKind<'ast> {
    IntLit(&'ast IntLitExpr<'ast>),
    FloatLit(&'ast FloatLitExpr<'ast>),
    StrLit(&'ast StrLitExpr<'ast>),
    CharLit(&'ast CharLitExpr<'ast>),
    BoolLit(&'ast BoolLitExpr<'ast>),
    Block(&'ast BlockExpr<'ast>),
    UnaryOp(&'ast UnaryOpExpr<'ast>),
    Borrow(&'ast BorrowExpr<'ast>),
    BinaryOp(&'ast BinaryOpExpr<'ast>),
    QuestionMark(&'ast QuestionMarkExpr<'ast>),
    As(&'ast AsExpr<'ast>),
    Path(&'ast PathExpr<'ast>),
    Call(&'ast CallExpr<'ast>),
    Array(&'ast ArrayExpr<'ast>),
    Tuple(&'ast TupleExpr<'ast>),
    Ctor(&'ast CtorExpr<'ast>),
    Range(&'ast RangeExpr<'ast>),
    Unstable(&'ast UnstableExpr<'ast>),
}

impl<'ast> ExprKind<'ast> {
    impl_expr_kind_fn!(span() -> &Span<'ast>);
    impl_expr_kind_fn!(id() -> ExprId);
    impl_expr_kind_fn!(ty() -> TyKind<'ast>);
    impl_expr_kind_fn!(precedence() -> ExprPrecedence);
}

#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprPrecedence {
    Lit = 0x1400_0000,
    Block = 0x1400_0001,
    Ctor = 0x1400_0002,

    Path = 0x1300_0000,

    Method = 0x1200_0000,
    Call = 0x1200_0001,

    Field = 0x1100_0000,

    Fn = 0x1000_0000,
    Index = 0x1000_0001,

    QuestionMark = 0x0F00_0000,

    /// The unary `-` operator
    Neg = 0x0E00_0000,
    /// The `!` operator
    Not = 0x0E00_0001,
    /// The unary `*` operator
    Deref = 0x0E00_0002,
    /// The unary `&` operator
    Reference = 0x0E00_0003,

    As = 0x0D00_0000,

    /// The binary `*` operator
    Mul = 0x0C00_0000,
    /// The `/` operator
    Div = 0x0C00_0001,
    /// The `%` operator
    Rem = 0x0C00_0002,

    /// The `+` operator
    Add = 0x0B00_0000,
    /// The binary `-` operator
    Sub = 0x0B00_0001,

    /// The `>>` operator
    Shr = 0x0A00_0000,
    /// The `<<` operator
    Shl = 0x0A00_0001,

    /// The binary `&` operator
    BitAnd = 0x0900_0000,

    /// The `^` operator
    BitXor = 0x0800_0000,

    /// The `|` operator
    BitOr = 0x0700_0000,

    /// The `==`, `!=`, `<`, `<=`, `>`, `>=` operators
    Comparison = 0x0600_0000,

    /// The `&&` operator
    And = 0x0500_0000,

    /// The `||` operator
    Or = 0x0400_0000,

    /// Ranges `0..10`, `0..=8`
    Range = 0x0300_0000,

    /// This precedence level includes compound assignment operators, like:
    /// `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
    AssignOp = 0x0200_0000,

    Closure = 0x0100_0000,
    Break = 0x0100_0001,
    Return = 0x0100_0002,
    /// The precedence originates from an unstable source. The stored value provides
    /// the current precedence of this expression. This might change in the future
    Unstable(i32),
}

macro_rules! impl_expr_kind_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_expr_kind_fn!($method() -> $return_ty,
            IntLit, FloatLit, StrLit, CharLit, BoolLit, Block, UnaryOp, Borrow,
            BinaryOp, QuestionMark, As, Path, Call, Array, Tuple, Ctor, Range,
            Unstable
        );
    };
    ($method:ident () -> $return_ty:ty $(, $kind:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $(ExprKind::$kind(data) => data.$method(),)*
            }
        }
    };
}

use impl_expr_kind_fn;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct CommonExprData<'ast> {
    /// The lifetime is not needed right now, but it's safer to include it for
    /// future additions. Having it in this struct makes it easier for all
    /// expression structs, as they will have a valid use for `'ast` even if they
    /// don't need it. Otherwise, we might need to declare this field in each
    /// expression.
    _lifetime: PhantomData<&'ast ()>,
    id: ExprId,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonExprData<'ast> {
    pub fn new(id: ExprId, span: SpanId) -> Self {
        Self {
            _lifetime: PhantomData,
            id,
            span,
        }
    }
}

macro_rules! impl_expr_data {
    ($self_ty:ty, $enum_name:ident) => {
        $crate::ast::expr::impl_expr_data!($self_ty, $enum_name,
            fn precedence(&self) -> $crate::ast::expr::ExprPrecedence {
                $crate::ast::expr::ExprPrecedence::$enum_name
            }
        );
    };
    ($self_ty:ty, $enum_name:ident, $precedence_fn:item) => {
        impl<'ast> super::ExprData<'ast> for $self_ty {
            fn id(&self) -> crate::ast::ExprId {
                self.data.id
            }

            fn span(&self) -> &crate::ast::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.get_span(self.data.span))
            }

            fn ty(&self) -> $crate::ast::ty::TyKind<'ast> {
                todo!("delegate ty request to driver")
            }

            $precedence_fn

            fn as_expr(&'ast self) -> crate::ast::expr::ExprKind<'ast> {
                $crate::ast::expr::ExprKind::$enum_name(self)
            }
        }

        impl<'ast> From<&'ast $self_ty> for $crate::ast::expr::ExprKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::ast::expr::ExprKind::$enum_name(from)
            }
        }
    };
}

use impl_expr_data;
