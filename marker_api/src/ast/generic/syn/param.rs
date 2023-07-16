use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ast::expr::ConstExpr;
use crate::ast::generic::{Lifetime, SynGenericParams};
use crate::ast::ty::SynTyKind;
use crate::ast::{GenericId, Span, SpanId, SymbolId};
use crate::context::with_cx;
use crate::ffi::{FfiOption, FfiSlice};
use crate::private::Sealed;

use super::SynTyParamBound;

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
/// Bounds declared with the parameter, like the `: Copy` in the example above,
/// are stored in the [`SynGenericParams`](`super::SynGenericParams`) of the item, that
/// introduced this parameter.
///
/// See: <https://doc.rust-lang.org/reference/items/generics.html>
#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum SynGenericParamKind<'ast> {
    Lifetime(&'ast SynLifetimeParam<'ast>),
    Ty(&'ast SynTyParam<'ast>),
    Const(&'ast SynConstParam<'ast>),
}

impl<'ast> SynGenericParamKind<'ast> {
    /// This returns the [`Span`], of the defined parameter, if this parameter originates from
    /// source code.
    pub fn span(&self) -> Option<&Span<'ast>> {
        match self {
            SynGenericParamKind::Lifetime(lt) => lt.span(),
            SynGenericParamKind::Ty(ty) => ty.span(),
            SynGenericParamKind::Const(con) => con.span(),
        }
    }

    pub fn id(&self) -> GenericId {
        match self {
            SynGenericParamKind::Ty(param) => param.id(),
            SynGenericParamKind::Lifetime(param) => param.id(),
            SynGenericParamKind::Const(param) => param.id(),
        }
    }
}

/// This trait is a collection of common information that is provided by all
/// generic parameters.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait SynGenericParamData<'ast>: Debug + Sealed {
    /// This returns the span, of the defined parameter, if this parameter originates from source
    /// code.
    fn span(&self) -> Option<&Span<'ast>>;

    // FIXME(xFrednet): Add `fn attrs() -> ??? {}`, see rust-marker/marker#51
}

/// A type parameter with optional bounds like `T` and `U` in this example:
///
/// ```
/// //     v
/// fn foo<T, U: Copy + 'static>() {}
/// //        ^^^^^^^^^^^^^^^^^
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct SynTyParam<'ast> {
    _data: PhantomData<&'ast ()>,
    id: GenericId,
    name: SymbolId,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SynTyParam<'ast> {
    pub fn new(span: Option<SpanId>, name: SymbolId, id: GenericId) -> Self {
        Self {
            _data: PhantomData,
            id,
            name,
            span: span.into(),
        }
    }
}

impl<'ast> SynTyParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }
}

impl<'ast> SynGenericParamData<'ast> for SynTyParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

impl Sealed for SynTyParam<'_> {}

impl<'ast> From<&'ast SynTyParam<'ast>> for SynGenericParamKind<'ast> {
    fn from(src: &'ast SynTyParam<'ast>) -> Self {
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
#[derive(Debug)]
pub struct SynLifetimeParam<'ast> {
    _data: PhantomData<&'ast ()>,
    id: GenericId,
    name: SymbolId,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SynLifetimeParam<'ast> {
    pub fn new(id: GenericId, name: SymbolId, span: Option<SpanId>) -> Self {
        Self {
            _data: PhantomData,
            id,
            name,
            span: span.into(),
        }
    }
}

impl<'ast> SynLifetimeParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }
}

impl<'ast> SynGenericParamData<'ast> for SynLifetimeParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

impl Sealed for SynLifetimeParam<'_> {}

impl<'ast> From<&'ast SynLifetimeParam<'ast>> for SynGenericParamKind<'ast> {
    fn from(src: &'ast SynLifetimeParam<'ast>) -> Self {
        Self::Lifetime(src)
    }
}

/// A constant parameter with an optional constant value, like this:
///
/// ```
/// struct Matrix<const N: usize = 3> {}
/// //            ^^^^^^^^^^^^^^^^^^
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct SynConstParam<'ast> {
    id: GenericId,
    name: SymbolId,
    ty: SynTyKind<'ast>,
    default: FfiOption<ConstExpr<'ast>>,
    span: SpanId,
}

impl<'ast> SynConstParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }

    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    pub fn default(&self) -> Option<&ConstExpr<'ast>> {
        self.default.get()
    }
}

impl Sealed for SynConstParam<'_> {}

impl<'ast> SynGenericParamData<'ast> for SynConstParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        Some(with_cx(self, |cx| cx.span(self.span)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynConstParam<'ast> {
    pub fn new(
        id: GenericId,
        name: SymbolId,
        ty: SynTyKind<'ast>,
        default: Option<ConstExpr<'ast>>,
        span: SpanId,
    ) -> Self {
        Self {
            id,
            name,
            ty,
            default: default.into(),
            span,
        }
    }
}

/// This represents a single clause in a [`where`](<https://doc.rust-lang.org/stable/reference/items/generics.html#where-clauses>) statement
///
/// ```
/// fn foo<'a, T>()
/// where
///     'a: 'static,
///     T: Iterator + 'a,
///     T::Item: Copy,
///     String: PartialEq<T>,
///     i32: Default,
/// {}
/// ```
#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum SynWhereClauseKind<'ast> {
    Lifetime(&'ast SynLifetimeClause<'ast>),
    Ty(&'ast SynTyClause<'ast>),
}

#[repr(C)]
#[derive(Debug)]
pub struct SynLifetimeClause<'ast> {
    lifetime: Lifetime<'ast>,
    bounds: FfiSlice<'ast, Lifetime<'ast>>,
}

impl<'ast> SynLifetimeClause<'ast> {
    pub fn lifetime(&self) -> &Lifetime<'ast> {
        &self.lifetime
    }

    pub fn bounds(&self) -> &[Lifetime<'ast>] {
        self.bounds.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynLifetimeClause<'ast> {
    pub fn new(lifetime: Lifetime<'ast>, bounds: &'ast [Lifetime<'ast>]) -> Self {
        Self {
            lifetime,
            bounds: bounds.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SynTyClause<'ast> {
    params: FfiOption<SynGenericParams<'ast>>,
    ty: SynTyKind<'ast>,
    bounds: FfiSlice<'ast, SynTyParamBound<'ast>>,
}

impl<'ast> SynTyClause<'ast> {
    /// Additional parameters introduced as a part of this where clause with a `for`.
    pub fn params(&self) -> Option<&SynGenericParams<'ast>> {
        self.params.get()
    }

    /// The bound type
    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    /// The bounds applied to the specified type.
    pub fn bounds(&self) -> &'ast [SynTyParamBound<'ast>] {
        self.bounds.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynTyClause<'ast> {
    pub fn new(
        params: Option<SynGenericParams<'ast>>,
        ty: SynTyKind<'ast>,
        bounds: &'ast [SynTyParamBound<'ast>],
    ) -> Self {
        Self {
            params: params.into(),
            ty,
            bounds: bounds.into(),
        }
    }
}
