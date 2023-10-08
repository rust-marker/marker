use std::fmt::Debug;
use std::marker::PhantomData;

use crate::{
    ast::{
        expr::ConstExpr,
        generic::{GenericParams, Lifetime},
        ty::TyKind,
    },
    common::{GenericId, SpanId, SymbolId},
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
    private::Sealed,
    span::Span,
};

use super::TyParamBound;

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
/// are stored in the [`GenericParams`](`super::GenericParams`) of the item, that
/// introduced this parameter.
///
/// See: <https://doc.rust-lang.org/reference/items/generics.html>
#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum GenericParamKind<'ast> {
    Lifetime(&'ast LifetimeParam<'ast>),
    Ty(&'ast TyParam<'ast>),
    Const(&'ast ConstParam<'ast>),
}

impl<'ast> GenericParamKind<'ast> {
    /// This returns the [`Span`], of the defined parameter, if this parameter originates from
    /// source code.
    pub fn span(&self) -> Option<&Span<'ast>> {
        match self {
            GenericParamKind::Lifetime(lt) => lt.span(),
            GenericParamKind::Ty(ty) => ty.span(),
            GenericParamKind::Const(con) => con.span(),
        }
    }

    pub fn id(&self) -> GenericId {
        match self {
            GenericParamKind::Ty(param) => param.id(),
            GenericParamKind::Lifetime(param) => param.id(),
            GenericParamKind::Const(param) => param.id(),
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
pub struct TyParam<'ast> {
    _data: PhantomData<&'ast ()>,
    id: GenericId,
    name: SymbolId,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TyParam<'ast> {
    pub fn new(span: Option<SpanId>, name: SymbolId, id: GenericId) -> Self {
        Self {
            _data: PhantomData,
            id,
            name,
            span: span.into(),
        }
    }
}

impl<'ast> TyParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }
}

impl<'ast> SynGenericParamData<'ast> for TyParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
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
#[derive(Debug)]
pub struct LifetimeParam<'ast> {
    _data: PhantomData<&'ast ()>,
    id: GenericId,
    name: SymbolId,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeParam<'ast> {
    pub fn new(id: GenericId, name: SymbolId, span: Option<SpanId>) -> Self {
        Self {
            _data: PhantomData,
            id,
            name,
            span: span.into(),
        }
    }
}

impl<'ast> LifetimeParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }
}

impl<'ast> SynGenericParamData<'ast> for LifetimeParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

impl<'ast> From<&'ast LifetimeParam<'ast>> for GenericParamKind<'ast> {
    fn from(src: &'ast LifetimeParam<'ast>) -> Self {
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
pub struct ConstParam<'ast> {
    id: GenericId,
    name: SymbolId,
    ty: TyKind<'ast>,
    default: FfiOption<ConstExpr<'ast>>,
    span: SpanId,
}

impl<'ast> ConstParam<'ast> {
    pub fn id(&self) -> GenericId {
        self.id
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }

    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    pub fn default(&self) -> Option<&ConstExpr<'ast>> {
        self.default.get()
    }
}

impl<'ast> SynGenericParamData<'ast> for ConstParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        Some(with_cx(self, |cx| cx.span(self.span)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstParam<'ast> {
    pub fn new(
        id: GenericId,
        name: SymbolId,
        ty: TyKind<'ast>,
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
pub enum WhereClauseKind<'ast> {
    Lifetime(&'ast LifetimeClause<'ast>),
    Ty(&'ast TyClause<'ast>),
}

#[repr(C)]
#[derive(Debug)]
pub struct LifetimeClause<'ast> {
    lifetime: Lifetime<'ast>,
    bounds: FfiSlice<'ast, Lifetime<'ast>>,
}

impl<'ast> LifetimeClause<'ast> {
    pub fn lifetime(&self) -> &Lifetime<'ast> {
        &self.lifetime
    }

    pub fn bounds(&self) -> &[Lifetime<'ast>] {
        self.bounds.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeClause<'ast> {
    pub fn new(lifetime: Lifetime<'ast>, bounds: &'ast [Lifetime<'ast>]) -> Self {
        Self {
            lifetime,
            bounds: bounds.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct TyClause<'ast> {
    params: FfiOption<GenericParams<'ast>>,
    ty: TyKind<'ast>,
    bounds: FfiSlice<'ast, TyParamBound<'ast>>,
}

impl<'ast> TyClause<'ast> {
    /// Additional parameters introduced as a part of this where clause with a `for`.
    pub fn params(&self) -> Option<&GenericParams<'ast>> {
        self.params.get()
    }

    /// The bound type
    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    /// The bounds applied to the specified type.
    pub fn bounds(&self) -> &'ast [TyParamBound<'ast>] {
        self.bounds.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyClause<'ast> {
    pub fn new(params: Option<GenericParams<'ast>>, ty: TyKind<'ast>, bounds: &'ast [TyParamBound<'ast>]) -> Self {
        Self {
            params: params.into(),
            ty,
            bounds: bounds.into(),
        }
    }
}
