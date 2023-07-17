use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ast::expr::ConstExpr;
use crate::ast::ty::SynTyKind;
use crate::ast::{GenericId, Span, SpanId, SymbolId};
use crate::context::with_cx;
use crate::ffi::FfiOption;
use crate::private::Sealed;

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
    Ty(&'ast TyParam<'ast>),
    Lifetime(&'ast LifetimeParam<'ast>),
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
pub trait GenericParamData<'ast>: Debug + Sealed {
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

impl<'ast> GenericParamData<'ast> for TyParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

impl Sealed for TyParam<'_> {}

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

impl<'ast> GenericParamData<'ast> for LifetimeParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

impl Sealed for LifetimeParam<'_> {}

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
    ty: SynTyKind<'ast>,
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

    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    pub fn default(&self) -> Option<&ConstExpr<'ast>> {
        self.default.get()
    }
}

impl Sealed for ConstParam<'_> {}

impl<'ast> GenericParamData<'ast> for ConstParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        Some(with_cx(self, |cx| cx.span(self.span)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstParam<'ast> {
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
