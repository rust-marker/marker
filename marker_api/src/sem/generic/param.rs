use crate::common::TyDefId;

use super::GenericArgs;

/// A semantic trait bound used by [`TraitTy`](`crate::sem::ty::TraitObjTy`)
#[repr(C)]
#[derive(Debug)]
pub struct TraitBound<'ast> {
    /// This is used for relaxed type bounds like `?Size`. This is probably not
    /// the best representation. Rustc uses a `TraitBoundModifier` enum which
    /// is interesting, but would only have two states right now.
    is_relaxed: bool,
    trait_id: TyDefId,
    trait_generic_args: GenericArgs<'ast>,
}

impl<'ast> TraitBound<'ast> {
    pub fn is_relaxed(&self) -> bool {
        self.is_relaxed
    }

    /// The [`TyDefId`] of the bound trait.
    pub fn trait_id(&self) -> TyDefId {
        self.trait_id
    }

    /// The [`GenericArgs`] used by the bound trait.
    pub fn trait_generic_args(&self) -> &GenericArgs<'ast> {
        &self.trait_generic_args
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitBound<'ast> {
    pub fn new(is_relaxed: bool, trait_id: TyDefId, trait_generic_args: GenericArgs<'ast>) -> Self {
        Self {
            is_relaxed,
            trait_id,
            trait_generic_args,
        }
    }
}
