use crate::ast::{Span, SpanId, TraitRef};
use crate::context::with_cx;

use super::Lifetime;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TyParamBound<'ast> {
    Lifetime(&'ast Lifetime<'ast>),
    TraitBound(&'ast TraitBound<'ast>),
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TraitBound<'ast> {
    /// This is used for relaxed type bounds like `?Size`. This is probably not
    /// the best representation. Rustc uses a `TraitBoundModifier` enum which
    /// is interesting, but would only have two states right now.
    is_relaxed: bool,
    trait_ref: TraitRef<'ast>,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitBound<'ast> {
    pub fn new(is_relaxed: bool, trait_ref: TraitRef<'ast>, span: SpanId) -> Self {
        Self {
            is_relaxed,
            trait_ref,
            span,
        }
    }
}

impl<'ast> TraitBound<'ast> {
    pub fn trait_ref(&self) -> &TraitRef<'ast> {
        &self.trait_ref
    }

    /// This returns true, when the bound is relaxed. This is currently only
    /// possible for the `Sized` trait by writing `?Sized`.
    // FIXME: I don't like the name of this function, but can't think of a
    // better name/representation for it.
    pub fn is_relaxed(&self) -> bool {
        self.is_relaxed
    }

    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }
}
