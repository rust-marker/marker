use crate::ast::{AstPath, ItemId, Span, SpanId, SymbolId};
use crate::context::AstContext;
use crate::ffi::{FfiOption, FfiSlice};

use super::{GenericArgs, GenericParamData, GenericParamKind, Lifetime};

#[repr(C)]
#[derive(Debug)]
pub struct TypeParam<'ast> {
    cx: &'ast AstContext<'ast>,
    span: FfiOption<SpanId>,
    ident: SymbolId,
    bounds: FfiSlice<'ast, &'ast TypeParamBound<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TypeParam<'ast> {
    pub fn new(
        cx: &'ast AstContext<'ast>,
        span: FfiOption<SpanId>,
        ident: SymbolId,
        bounds: FfiSlice<'ast, &'ast TypeParamBound<'ast>>,
    ) -> Self {
        Self {
            cx,
            span,
            ident,
            bounds,
        }
    }
}

impl<'ast> TypeParam<'ast> {
    pub fn ident(&self) -> String {
        self.cx.symbol_str(self.ident)
    }

    pub fn bounds(&self) -> &[&TypeParamBound<'ast>] {
        self.bounds.get()
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|sym| self.cx.get_span(*sym))
    }
}

impl<'ast> GenericParamData<'ast> for TypeParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| self.cx.get_span(*span))
    }
}

impl<'ast> From<&'ast TypeParam<'ast>> for GenericParamKind<'ast> {
    fn from(src: &'ast TypeParam<'ast>) -> Self {
        Self::Type(src)
    }
}

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum TypeParamBound<'ast> {
    Lifetime(&'ast Lifetime<'ast>),
    TraitBound(&'ast TraitBound<'ast>),
}

// FIXME: Add support for higher order traits thingies with `for<'a>` and other magic.
#[repr(C)]
#[derive(Debug)]
pub struct TraitBound<'ast> {
    cx: &'ast AstContext<'ast>,
    /// This is used for relaxed type bounds like `?Size`. This is probably not
    /// the best representation. Rustc uses a `TraitBoundModifier` enum which
    /// is interesting, but would only have two states right now.
    is_relaxed: bool,
    trait_ref: TraitRef<'ast>,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitBound<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, is_relaxed: bool, trait_ref: TraitRef<'ast>, span: SpanId) -> Self {
        Self {
            cx,
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
        self.cx.get_span(self.span)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct TraitRef<'ast> {
    path: AstPath<'ast>,
    item_id: ItemId,
    generics: GenericArgs<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitRef<'ast> {
    pub fn new(path: AstPath<'ast>, item_id: ItemId, generics: GenericArgs<'ast>) -> Self {
        Self {
            path,
            item_id,
            generics,
        }
    }
}

impl<'ast> TraitRef<'ast> {
    pub fn path(&self) -> &AstPath<'ast> {
        &self.path
    }

    pub fn trait_id(&self) -> ItemId {
        self.item_id
    }

    pub fn generics(&self) -> &GenericArgs<'ast> {
        &self.generics
    }
}
