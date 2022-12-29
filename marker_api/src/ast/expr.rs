use super::{ty::TyKind, ExprId, Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

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
mod unstable_expr;
pub use unstable_expr::*;

pub trait ExprData<'ast>: Debug {
    fn id(&self) -> ExprId;

    /// Returns the span of this pattern.
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
    Unstable(&'ast UnstableExpr<'ast>),
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprPrecedence {
    Lit,
    /// The precedents originates from an unstable source. The stored value provides
    /// the current precedence of this expression. This is open to change
    Unstable(i32),
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct CommonExprData<'ast> {
    /// The lifetime is not needed right now, but it's safer to include it for
    /// future additions. Having it in this struct makes it easier for all
    /// pattern structs, as they will have a valid use for `'ast` even if they
    /// don't need it. Otherwise, we might need to declare this field in each
    /// pattern.
    _lifetime: PhantomData<&'ast ()>,
    id: ExprId,
    span: SpanId,
}

macro_rules! impl_expr_data {
    ($self_ty:ty, $enum_name:ident) => {
        impl_expr_data!($self_ty, $enum_name,
            fn precedence(&self) -> ExprPrecedence {
                $crate::ast::expr::ExprPrecedence::$enum_name
            }
        )
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
