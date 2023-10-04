use crate::{
    ast::{expr::ConstExpr, ty::TyKind, TraitRef},
    common::{SpanId, SymbolId},
    context::with_cx,
    span::Span,
};

use super::Lifetime;

/// The syntactic representation of a generic argument, like this:
///
/// ```
/// let _bar: Vec<String> = vec![];
/// //            ^^^^^^
/// ```
#[repr(C)]
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub struct LifetimeArg<'ast> {
    lifetime: Lifetime<'ast>,
}

impl<'ast> LifetimeArg<'ast> {
    pub fn lifetime(&self) -> &Lifetime<'ast> {
        &self.lifetime
    }

    /// The [`Span`] of the type argument.
    pub fn span(&self) -> &Span<'ast> {
        self.lifetime
            .span()
            .expect("every lifetime inside syntactic `SynLifetimeArg` should have a span")
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeArg<'ast> {
    pub fn new(lifetime: Lifetime<'ast>) -> Self {
        Self { lifetime }
    }
}

/// The syntactic representation of a generic argument, like this:
///
/// ```
/// let _bar: Vec<String> = vec![];
/// //            ^^^^^^
/// ```
#[repr(C)]
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub struct TyArg<'ast> {
    ty: TyKind<'ast>,
}

impl<'ast> TyArg<'ast> {
    pub fn ty(&self) -> TyKind<'_> {
        self.ty
    }

    /// The [`Span`] of the type argument.
    pub fn span(&self) -> &Span<'ast> {
        self.ty.span()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyArg<'ast> {
    pub fn new(ty: TyKind<'ast>) -> Self {
        Self { ty }
    }
}

/// A generic bound in form `<identifier=type>`. For example, `Item=i32` would be
/// the generic binding here:
///
/// ```ignore
/// let _baz: &dyn Iterator<Item=i32> = todo!();
/// //                      ^^^^^^^^
/// ```
///
/// The corresponding instance would provide the name (`Item`), the defined type
/// (`i32`) and potentially the [`Span`] if this bound originates from source code.
///
/// See [paths in expressions](https://doc.rust-lang.org/reference/paths.html#paths-in-expressions)
/// for more information.
#[repr(C)]
#[derive(Debug)]
pub struct BindingArg<'ast> {
    span: SpanId,
    ident: SymbolId,
    ty: TyKind<'ast>,
}

impl<'ast> BindingArg<'ast> {
    /// The name of the identifier used in the binding. For example:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=i32> = todo!();
    /// //                      ^^^^
    /// ```
    ///
    /// Would return `Item` as the identifier.
    pub fn ident(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    /// The type that the identifier is set to. For example:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=i32> = todo!();
    /// //                           ^^^
    /// ```
    ///
    /// Would return `i32` as the type.
    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    /// The [`Span`] of the binding.
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> BindingArg<'ast> {
    pub fn new(span: SpanId, ident: SymbolId, ty: TyKind<'ast>) -> Self {
        Self { span, ident, ty }
    }
}

/// A constant expression as an argument for a constant generic.
///
/// ```
/// struct Vec<const N: usize> {
///     data: [f32; N],
/// }
///
/// // An integer literal as a const generic argument
/// //               v
/// fn vec3() -> Vec<3> {
///     // [...]
///     # todo!()
/// }
///
/// // A const generic parameter as an const generic argument
/// //                       v
/// impl<const N: usize> Vec<N> {
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct ConstArg<'ast> {
    span: SpanId,
    expr: ConstExpr<'ast>,
}

impl<'ast> ConstArg<'ast> {
    /// The [`ConstExpr`] that is used as an argument.
    pub fn expr(&self) -> &ConstExpr<'ast> {
        &self.expr
    }

    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstArg<'ast> {
    pub fn new(span: SpanId, expr: ConstExpr<'ast>) -> Self {
        Self { span, expr }
    }
}

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum TyParamBound<'ast> {
    Lifetime(&'ast Lifetime<'ast>),
    TraitBound(&'ast TraitBound<'ast>),
}

#[repr(C)]
#[derive(Debug)]
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
        with_cx(self, |cx| cx.span(self.span))
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
