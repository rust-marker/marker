use super::{Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

mod ident_pat;
pub use ident_pat::*;
mod wildcard_pat;
pub use wildcard_pat::*;
mod rest_pat;
pub use rest_pat::*;
mod ref_pat;
pub use ref_pat::*;
mod struct_pat;
pub use struct_pat::*;
mod tuple_pat;
pub use tuple_pat::*;
mod slice_pat;
pub use slice_pat::*;
mod or_pat;
pub use or_pat::*;
mod unstable_pat;
pub use unstable_pat::*;

pub trait PatData<'ast>: Debug {
    /// Returns the span of this pattern.
    fn span(&self) -> &Span<'ast>;

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
    Unstable(&'ast UnstablePat<'ast>),
}

impl<'ast> PatKind<'ast> {
    impl_pat_data_fn!(span() -> &Span<'ast>);
}

macro_rules! impl_pat_data_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_pat_data_fn!(
            $method() -> $return_ty,
            Ident, Wildcard, Rest, Ref, Struct, Tuple, Slice, Or, Unstable
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

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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
                $crate::context::with_cx(self, |cx| cx.get_span(self.data.span))
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
    };
}

use impl_pat_data;
