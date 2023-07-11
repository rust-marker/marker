use std::marker::PhantomData;

use crate::{
    ast::{
        ty::{SemTyKind, SynTyKind},
        GenericId, ItemId, Span, SpanId, SymbolId,
    },
    context::with_cx,
    ffi::FfiOption,
};

/// A lifetime used as a generic argument or on a reference like this:
///
/// ```
/// # use core::marker::PhantomData;
/// # #[derive(Default)]
/// # struct Item<'a> {
/// #     _data: PhantomData<&'a ()>,
/// # }
///
/// # fn example<'a>() {
/// let _item: Item<'_> = Item::default();
/// //              ^^
/// let _item: Item<'a> = Item::default();
/// //              ^^
/// let _item: &'static str = "Hello world";
/// //          ^^^^^^^
/// # }
/// ```
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Lifetime<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    span: FfiOption<SpanId>,
    kind: LifetimeKind,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) enum LifetimeKind {
    /// A lifetime with a label like `'ast`
    Label(SymbolId, GenericId),
    /// The magic `'static` lifetime
    Static,
    /// The mysterious `'_` lifetime
    Infer,
}

#[cfg(feature = "driver-api")]
impl<'ast> Lifetime<'ast> {
    pub fn new(span: Option<SpanId>, kind: LifetimeKind) -> Self {
        Self {
            _lifetime: PhantomData,
            span: span.into(),
            kind,
        }
    }
}

impl<'ast> Lifetime<'ast> {
    /// This returns the [`GenericId`] of this lifetime, if it's labeled, or [`None`]
    /// otherwise. `'static` will also return [`None`]
    pub fn id(&self) -> Option<GenericId> {
        match self.kind {
            LifetimeKind::Label(_, id) => Some(id),
            _ => None,
        }
    }

    /// Note that the `'static` lifetime is not a label and will therefore return [`None`]
    pub fn label(&self) -> Option<&str> {
        match self.kind {
            LifetimeKind::Label(sym, _) => Some(with_cx(self, |cx| cx.symbol_str(sym))),
            _ => None,
        }
    }

    pub fn has_label(&self) -> bool {
        matches!(self.kind, LifetimeKind::Label(..))
    }

    pub fn is_static(&self) -> bool {
        matches!(self.kind, LifetimeKind::Static)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self.kind, LifetimeKind::Infer)
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BindingGenericArg<'ast> {
    span: FfiOption<SpanId>,
    ident: SymbolId,
    ty: SynTyKind<'ast>,
}

impl<'ast> BindingGenericArg<'ast> {
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
    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    /// The [`Span`] of the binding, if this instance originates from source code.
    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> BindingGenericArg<'ast> {
    pub fn new(span: Option<SpanId>, ident: SymbolId, ty: SynTyKind<'ast>) -> Self {
        Self {
            span: span.into(),
            ident,
            ty,
        }
    }
}

/// A semantic generic bound in the form `<identifier=type>`. For example,
/// `Item=i32` would be the generic binding here:
///
/// ```ignore
/// let _baz: &dyn Iterator<Item=i32> = todo!();
/// //                      ^^^^^^^^
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct SemTyBindingArg<'ast> {
    binding_target: ItemId,
    ty: SemTyKind<'ast>,
}

impl<'ast> SemTyBindingArg<'ast> {
    /// This returns the `ItemId` of the binding target.
    pub fn binding_target(&self) -> ItemId {
        self.binding_target
    }

    /// The type that the binding is set to. For example:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=i32> = todo!();
    /// //                           ^^^
    /// ```
    ///
    /// Would return `i32` as the type.
    pub fn ty(&self) -> SemTyKind<'ast> {
        self.ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTyBindingArg<'ast> {
    pub fn new(binding_target: ItemId, ty: SemTyKind<'ast>) -> Self {
        Self { binding_target, ty }
    }
}
