use std::fmt::Debug;
use std::marker::PhantomData;

mod syn;
pub use syn::*;
mod sem;
pub use sem::*;

use crate::{
    ast::{GenericId, SpanId, SymbolId},
    context::with_cx,
    ffi::FfiOption,
    span::Span,
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
#[derive(Debug)]
pub struct Lifetime<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    span: FfiOption<SpanId>,
    kind: LifetimeKind,
}

#[repr(C)]
#[derive(Debug)]
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
