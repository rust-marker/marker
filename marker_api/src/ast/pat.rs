use crate::private::Sealed;

use super::{Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

mod ident_pat;
mod lit_pat;
mod or_pat;
mod path_pat;
mod place_pat;
mod range_pat;
mod ref_pat;
mod rest_pat;
mod slice_pat;
mod struct_pat;
mod tuple_pat;
mod unstable_pat;
mod wildcard_pat;
pub use ident_pat::*;
pub use lit_pat::*;
pub use or_pat::*;
pub use path_pat::*;
pub use place_pat::*;
pub use range_pat::*;
pub use ref_pat::*;
pub use rest_pat::*;
pub use slice_pat::*;
pub use struct_pat::*;
pub use tuple_pat::*;
pub use unstable_pat::*;
pub use wildcard_pat::*;

/// This trait combines methods, which all patterns have in common.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait PatData<'ast>: Debug + Sealed {
    /// Returns the [`Span`] of this pattern.
    fn span(&self) -> &Span<'ast>;
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
    /// Patterns are used as assignees in [`AssignExpr`](crate::ast::expr::AssignExpr)
    /// nodes. Assign expressions can target place expressions like
    /// variables, [`IndexExpr`](crate::ast::expr::IndexExpr)s and
    /// [`FieldExpr`](crate::ast::expr::FieldExpr)s. These expressions would
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
    /// //   ^  ^^^ Place expressions nested in a tuple pattern
    /// ```
    ///
    /// Place expressions can currently only occur as targets in
    /// [`AssignExpr`](crate::ast::expr::AssignExpr)s. Patterns from
    /// [`LetStmts`](crate::ast::stmt::LetStmt)s and arguments in
    /// [`FnItem`](crate::ast::item::FnItem) will never contain place expressions.
    /// Static paths identifying [`ConstItem`](crate::ast::item::ConstItem)s or
    /// [`EnumItem`](crate::ast::item::EnumItem) variants are expressed with the
    /// [`PatKind::Path`] variant.
    Place(&'ast PlacePat<'ast>),
    Lit(&'ast LitPat<'ast>),
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
                $(PatKind::$item(data, ..) => data.$method(),)*
            }
        }
    };
}

use impl_pat_data_fn;

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
