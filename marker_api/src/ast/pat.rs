use crate::private::Sealed;

use super::{
    expr::{ExprKind, LitExprKind},
    Span, SpanId,
};

use std::{fmt::Debug, marker::PhantomData};

mod ident_pat;
mod or_pat;
mod path_pat;
mod range_pat;
mod ref_pat;
mod rest_pat;
mod slice_pat;
mod struct_pat;
mod tuple_pat;
mod unstable_pat;
mod wildcard_pat;
pub use ident_pat::*;
pub use or_pat::*;
pub use path_pat::*;
pub use range_pat::*;
pub use ref_pat::*;
pub use rest_pat::*;
pub use slice_pat::*;
pub use struct_pat::*;
pub use tuple_pat::*;
pub use unstable_pat::*;
pub use wildcard_pat::*;

/// This trait combines methods, which are common between all patterns.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait PatData<'ast>: Debug + Sealed {
    /// Returns the [`Span`] of this pattern.
    fn span(&self) -> &Span<'ast>;

    /// Returns this expression wrapped in it's [`PatKind`] variant.
    ///
    /// In function parameters, it's recommended to use `Into<PatKind<'ast>>`
    /// as a bound to support all patterns and `PatKind<'ast>` as parameters.
    fn as_pat(&'ast self) -> PatKind<'ast>;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum PatKind<'ast> {
    Ident(&'ast IdentPat<'ast>),
    Wildcard(&'ast WildcardPat<'ast>),
    Rest(&'ast RestPat<'ast>),
    Ref(&'ast RefPat<'ast>),
    Struct(&'ast StructPat<'ast>),
    Tuple(&'ast TuplePat<'ast>),
    Slice(&'ast SlicePat<'ast>),
    Or(&'ast OrPat<'ast>),
    /// Patterns are used as assignees in [`AssignExpr`](super::expr::AssignExpr)
    /// nodes. Assign expressions can target place expressions like
    /// variables, [`IndexExpr`](super::expr::IndexExpr)s and
    /// [`FieldExpr`](super::expr::FieldExpr)s. These expressions would
    /// be stored as this variant.
    ///
    /// ```
    /// # fn some_fn() -> (i32, i32) { (4, 5) }
    /// # let mut a = 1;
    /// # let mut b = (2, 3);
    /// # let mut c = [4, 5];
    ///     a = 6;
    /// //  ^ A path expression targeting the local variable `a`
    ///
    ///     b.1 = 7;
    /// //  ^^^ A field expression accessing field 1 on the local variable `b`
    ///
    ///     c[0] = 8;
    /// //  ^^^^ An index expression on local variable `c`
    ///
    ///     (a, b.0) = some_fn();
    /// //  ^^^^^^^^ Place expressions nested in a tuple pattern
    /// ```
    ///
    /// Place expressions can currently only occur as targets in
    /// [`AssignExpr`](super::expr::AssignExpr)s. Patterns from
    /// [`LetStmts`](super::stmt::LetStmt)s and arguments in
    /// [`FnItem`](super::item::FnItem) will never contain place expressions.
    /// Static paths identifying [`ConstItem`](super::item::ConstItem)s or
    /// [`EnumItem`](super::item::EnumItem) variants are expressed with the
    /// [`PatKind::Path`] variant.
    Place(ExprKind<'ast>),
    Lit(LitExprKind<'ast>),
    Path(&'ast PathPat<'ast>),
    Range(&'ast RangePat<'ast>),
    Unstable(&'ast UnstablePat<'ast>),
}

impl<'ast> PatKind<'ast> {
    impl_pat_data_fn!(span() -> &Span<'ast>);
}

macro_rules! impl_pat_data_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_pat_data_fn!(
            $method() -> $return_ty,
            Ident, Wildcard, Rest, Ref, Struct, Tuple, Slice, Or, Place, Lit, Range, Path, Unstable
        );
    };
    ($method:ident () -> $return_ty:ty $(, $item:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $(PatKind::$item(data) => data.$method(),)*
            }
        }
    };
}

use impl_pat_data_fn;

impl<'ast> PatData<'ast> for ExprKind<'ast> {
    fn span(&self) -> &Span<'ast> {
        self.span()
    }

    fn as_pat(&'ast self) -> PatKind<'ast> {
        PatKind::Place(*self)
    }
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct CommonPatData<'ast> {
    /// The lifetime is not needed right now, but it's safer to include it for
    /// future additions. Having it in this struct makes it easier for all
    /// pattern structs, as they will have a valid use for `'ast` even if they
    /// don't need it. Otherwise, we might need to declare this field in each
    /// pattern.
    _lifetime: PhantomData<&'ast ()>,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonPatData<'ast> {
    pub fn new(span: SpanId) -> Self {
        Self {
            _lifetime: PhantomData,
            span,
        }
    }
}

macro_rules! impl_pat_data {
    ($self_ty:ty, $enum_name:ident) => {
        impl<'ast> super::PatData<'ast> for $self_ty {
            fn span(&self) -> &crate::ast::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.span(self.data.span))
            }

            fn as_pat(&'ast self) -> crate::ast::pat::PatKind<'ast> {
                $crate::ast::pat::PatKind::$enum_name(self)
            }
        }

        impl<'ast> From<&'ast $self_ty> for $crate::ast::pat::PatKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::ast::pat::PatKind::$enum_name(from)
            }
        }

        impl<'ast> $crate::private::Sealed for $self_ty {}
    };
}

use impl_pat_data;
