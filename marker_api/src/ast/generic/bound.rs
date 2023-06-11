use crate::ast::{
    generic::SemGenericArgs,
    {Span, SpanId, TraitRef, TyDefId},
};
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

/// A semantic trait bound used by [`SemTraitTy`](`crate::ast::ty::SemTraitObjTy`)
#[repr(C)]
#[derive(Debug)]
pub struct SemTraitBound<'ast> {
    /// This is used for relaxed type bounds like `?Size`. This is probably not
    /// the best representation. Rustc uses a `TraitBoundModifier` enum which
    /// is interesting, but would only have two states right now.
    is_relaxed: bool,
    trait_id: TyDefId,
    trait_generic_args: SemGenericArgs<'ast>,
}

impl<'ast> SemTraitBound<'ast> {
    pub fn is_relaxed(&self) -> bool {
        self.is_relaxed
    }

    /// The [`TyDefId`] of the bound trait.
    pub fn trait_id(&self) -> TyDefId {
        self.trait_id
    }

    /// The [`SemGenericArgs`] used by the bound trait.
    pub fn trait_generic_args(&self) -> &SemGenericArgs<'ast> {
        &self.trait_generic_args
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTraitBound<'ast> {
    pub fn new(is_relaxed: bool, trait_id: TyDefId, trait_generic_args: SemGenericArgs<'ast>) -> Self {
        Self {
            is_relaxed,
            trait_id,
            trait_generic_args,
        }
    }
}
