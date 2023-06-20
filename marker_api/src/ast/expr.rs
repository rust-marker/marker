use super::{ty::SemTyKind, ExprId, Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

mod block_expr;
mod call_exprs;
mod control_flow_expr;
mod ctor_expr;
mod lit_expr;
mod op_exprs;
mod path_expr;
mod place_expr;
mod unstable_expr;
pub use block_expr::*;
pub use call_exprs::*;
pub use control_flow_expr::*;
pub use ctor_expr::*;
pub use lit_expr::*;
pub use op_exprs::*;
pub use path_expr::*;
pub use place_expr::*;
pub use unstable_expr::*;

pub trait ExprData<'ast>: Debug {
    fn id(&self) -> ExprId;

    /// Returns the span of this expression.
    fn span(&self) -> &Span<'ast>;

    // This returns the semantic type of this expression
    fn ty(&self) -> SemTyKind<'ast>;

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
    Closure(&'ast ClosureExpr<'ast>),
    UnaryOp(&'ast UnaryOpExpr<'ast>),
    Ref(&'ast RefExpr<'ast>),
    BinaryOp(&'ast BinaryOpExpr<'ast>),
    QuestionMark(&'ast QuestionMarkExpr<'ast>),
    Assign(&'ast AssignExpr<'ast>),
    As(&'ast AsExpr<'ast>),
    Path(&'ast PathExpr<'ast>),
    Call(&'ast CallExpr<'ast>),
    Method(&'ast MethodExpr<'ast>),
    Array(&'ast ArrayExpr<'ast>),
    Tuple(&'ast TupleExpr<'ast>),
    Ctor(&'ast CtorExpr<'ast>),
    Range(&'ast RangeExpr<'ast>),
    Index(&'ast IndexExpr<'ast>),
    Field(&'ast FieldExpr<'ast>),
    If(&'ast IfExpr<'ast>),
    Let(&'ast LetExpr<'ast>),
    Match(&'ast MatchExpr<'ast>),
    Break(&'ast BreakExpr<'ast>),
    Return(&'ast ReturnExpr<'ast>),
    Continue(&'ast ContinueExpr<'ast>),
    For(&'ast ForExpr<'ast>),
    Loop(&'ast LoopExpr<'ast>),
    While(&'ast WhileExpr<'ast>),
    Unstable(&'ast UnstableExpr<'ast>),
}

impl<'ast> ExprKind<'ast> {
    impl_expr_kind_fn!(ExprKind: span() -> &Span<'ast>);
    impl_expr_kind_fn!(ExprKind: id() -> ExprId);
    impl_expr_kind_fn!(ExprKind: ty() -> SemTyKind<'ast>);
    impl_expr_kind_fn!(ExprKind: precedence() -> ExprPrecedence);
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum LitExprKind<'ast> {
    Int(&'ast IntLitExpr<'ast>),
    Float(&'ast FloatLitExpr<'ast>),
    Str(&'ast StrLitExpr<'ast>),
    Char(&'ast CharLitExpr<'ast>),
    Bool(&'ast BoolLitExpr<'ast>),
}

impl<'ast> LitExprKind<'ast> {
    impl_expr_kind_fn!(LitExprKind: span() -> &Span<'ast>);
    impl_expr_kind_fn!(LitExprKind: id() -> ExprId);
    impl_expr_kind_fn!(LitExprKind: ty() -> SemTyKind<'ast>);
    impl_expr_kind_fn!(LitExprKind: precedence() -> ExprPrecedence);
}

impl<'ast> From<LitExprKind<'ast>> for ExprKind<'ast> {
    fn from(value: LitExprKind<'ast>) -> Self {
        match value {
            LitExprKind::Int(expr) => ExprKind::IntLit(expr),
            LitExprKind::Float(expr) => ExprKind::FloatLit(expr),
            LitExprKind::Str(expr) => ExprKind::StrLit(expr),
            LitExprKind::Char(expr) => ExprKind::CharLit(expr),
            LitExprKind::Bool(expr) => ExprKind::BoolLit(expr),
        }
    }
}

impl<'ast> TryFrom<ExprKind<'ast>> for LitExprKind<'ast> {
    type Error = ();

    fn try_from(value: ExprKind<'ast>) -> Result<Self, Self::Error> {
        match value {
            ExprKind::IntLit(expr) => Ok(LitExprKind::Int(expr)),
            ExprKind::FloatLit(expr) => Ok(LitExprKind::Float(expr)),
            ExprKind::StrLit(expr) => Ok(LitExprKind::Str(expr)),
            ExprKind::CharLit(expr) => Ok(LitExprKind::Char(expr)),
            ExprKind::BoolLit(expr) => Ok(LitExprKind::Bool(expr)),
            _ => Err(()),
        }
    }
}

#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprPrecedence {
    Lit = 0x1400_0000,
    Block = 0x1400_0001,
    Ctor = 0x1400_0002,
    Assign = 0x1400_0003,
    For = 0x1400_0004,
    Loop = 0x1400_0005,
    While = 0x1400_0006,

    Path = 0x1300_0000,

    Method = 0x1200_0000,
    Call = 0x1200_0001,
    // These three are just a guess, as they're not listed in the precedence table
    If = 0x1200_0002,
    Let = 0x1200_0003,
    Match = 0x1200_0004,

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
    Ref = 0x0E00_0003,

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
    Continue = 0x0100_0003,
    /// The precedence originates from an unstable source. The stored value provides
    /// the current precedence of this expression. This might change in the future
    Unstable(i32),
}

macro_rules! impl_expr_kind_fn {
    (ExprKind: $method:ident () -> $return_ty:ty) => {
        impl_expr_kind_fn!((ExprKind) $method() -> $return_ty,
            IntLit, FloatLit, StrLit, CharLit, BoolLit,
            Block, Closure,
            UnaryOp, Ref, BinaryOp, QuestionMark, As, Assign,
            Path, Index, Field,
            Call, Method,
            Array, Tuple, Ctor, Range,
            If, Let, Match, Break, Return, Continue, For, Loop, While,
            Unstable
        );
    };
    (LitExprKind: $method:ident () -> $return_ty:ty) => {
        impl_expr_kind_fn!((LitExprKind) $method() -> $return_ty,
            Int, Float, Str, Char, Bool
        );
    };
    (($self:ident) $method:ident () -> $return_ty:ty $(, $kind:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $($self::$kind(data) => data.$method(),)*
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

            fn ty(&self) -> $crate::ast::ty::SemTyKind<'ast> {
                $crate::context::with_cx(self, |cx| cx.expr_ty(self.data.id))
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
