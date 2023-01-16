use super::{ty::TyKind, ExprId, Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

// Literal expressions
mod int_lit_expr;
pub use int_lit_expr::*;
mod float_lit_expr;
pub use float_lit_expr::*;
mod str_lit_expr;
pub use str_lit_expr::*;
mod char_lit_expr;
pub use char_lit_expr::*;
mod bool_lit_expr;
pub use bool_lit_expr::*;
// other expressions
mod block_expr;
pub use block_expr::*;
mod unstable_expr;
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
    Unstable(&'ast UnstableExpr<'ast>),
}

impl<'ast> ExprKind<'ast> {
    impl_expr_kind_fn!(span() -> &Span<'ast>);
    impl_expr_kind_fn!(id() -> ExprId);
    impl_expr_kind_fn!(ty() -> TyKind<'ast>);
    impl_expr_kind_fn!(precedence() -> ExprPrecedence);
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprPrecedence {
    Block,
    Lit,
    /// The precedence originates from an unstable source. The stored value provides
    /// the current precedence of this expression. This might change in the future
    Unstable(i32),
}

macro_rules! impl_expr_kind_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_expr_kind_fn!($method() -> $return_ty,
            IntLit, FloatLit, StrLit, CharLit, BoolLit, Block, Unstable
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
