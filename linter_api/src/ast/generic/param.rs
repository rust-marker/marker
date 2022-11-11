use crate::ast::{GenericId, Span, SpanId, SymbolId};
use crate::context::with_cx;
use crate::ffi::{FfiOption, FfiSlice};

use super::{Lifetime, TyParamBound};

/// A singular generic parameter, like `'a` and `T` in this example:
///
/// ```
/// # use std::marker::PhantomData;
/// //          vv
/// struct Item<'a, T: Copy> {
/// //              ^
/// // ..
/// #    _data: PhantomData<&'a T>,
/// }
/// ```
///
/// See: <https://doc.rust-lang.org/reference/items/generics.html>
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GenericParamKind<'ast> {
    Ty(&'ast TyParam<'ast>),
    Lifetime(&'ast LifetimeParam<'ast>),
    // FIXME: Add const `ConstParam`
}

impl<'ast> GenericParamKind<'ast> {
    /// This returns the span, of the defined parameter, if this parameter is
    /// part of the code base.
    pub fn span(&self) -> Option<&Span<'ast>> {
        match self {
            GenericParamKind::Lifetime(lt) => lt.span(),
            GenericParamKind::Ty(ty) => ty.span(),
        }
    }

    pub fn id(&self) -> GenericId {
        match self {
            GenericParamKind::Ty(param) => param.id(),
            GenericParamKind::Lifetime(param) => param.id(),
        }
    }
}

/// This trait is a collection of common information that is provided by all
/// generic parameters.
pub trait GenericParamData<'ast> {
    /// This returns the span, of the defined parameter, if this parameter is
    /// part of the code base.
    fn span(&self) -> Option<&Span<'ast>>;
    // FIXME: Add `fn attrs(&self) -> &[Attrs<'ast>]` once implemented.
}

/// A type parameter with optional bounds like `T` and `U` in this example:
///
/// ```
/// //     v
/// fn foo<T, U: Copy + 'static>() {}
/// //        ^^^^^^^^^^^^^^^^^
/// ```
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TyParam<'ast> {
    id: GenericId,
    name: SymbolId,
    bounds: FfiSlice<'ast, TyParamBound<'ast>>,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TyParam<'ast> {
    pub fn new(span: Option<SpanId>, name: SymbolId, id: GenericId, bounds: &'ast [TyParamBound<'ast>]) -> Self {
        Self {
            id,
            name,
            bounds: bounds.into(),
            span: span.into(),
        }
    }
}

impl<'ast> TyParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.get_span(*span)))
    }
}

impl<'ast> GenericParamData<'ast> for TyParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.get_span(*span)))
    }
}

impl<'ast> From<&'ast TyParam<'ast>> for GenericParamKind<'ast> {
    fn from(src: &'ast TyParam<'ast>) -> Self {
        Self::Ty(src)
    }
}

/// A lifetime parameter like the `'long` and `'short: 'long` in:
///
/// ```
/// //     vvvvv
/// fn foo<'long, 'short: 'long>() {}
/// //             ^^^^^^^^^^^^
/// ```
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LifetimeParam<'ast> {
    id: GenericId,
    name: SymbolId,
    bounds: FfiSlice<'ast, Lifetime<'ast>>,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeParam<'ast> {
    pub fn new(id: GenericId, name: SymbolId, bounds: &'ast [Lifetime<'ast>], span: Option<SpanId>) -> Self {
        Self {
            id,
            name,
            bounds: bounds.into(),
            span: span.into(),
        }
    }
}

impl<'ast> LifetimeParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }

    /// Returns the bounds defined as part of the parameter. For example `'long`
    /// would be returned as the bound for `'short`.
    ///
    /// ```
    /// fn foo<'long, 'short: 'long>() {}
    /// //             ^^^^^^^^^^^^
    /// ```
    pub fn bounds(&self) -> &'ast [Lifetime<'ast>] {
        self.bounds.get()
    }
}

impl<'ast> GenericParamData<'ast> for LifetimeParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.get_span(*span)))
    }
}

impl<'ast> From<&'ast LifetimeParam<'ast>> for GenericParamKind<'ast> {
    fn from(src: &'ast LifetimeParam<'ast>) -> Self {
        Self::Lifetime(src)
    }
}
