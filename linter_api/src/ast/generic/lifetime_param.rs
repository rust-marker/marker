use std::marker::PhantomData;

use crate::ast::{Span, SpanId, SymbolId};
use crate::context::{with_cx, AstContext};
use crate::ffi::{FfiOption, FfiSlice};

use super::{GenericParamData, GenericParamKind};

#[repr(C)]
#[derive(Debug)]
pub struct LifetimeParam<'ast> {
    cx: &'ast AstContext<'ast>,
    span: FfiOption<SpanId>,
    lifetime: Lifetime<'ast>,
    bounds: FfiOption<LifetimeBounds<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeParam<'ast> {
    pub fn new(
        cx: &'ast AstContext<'ast>,
        span: FfiOption<SpanId>,
        lifetime: Lifetime<'ast>,
        bounds: FfiOption<LifetimeBounds<'ast>>,
    ) -> Self {
        Self {
            cx,
            span,
            lifetime,
            bounds,
        }
    }
}

impl<'ast> LifetimeParam<'ast> {
    pub fn lifetime(&self) -> &Lifetime<'ast> {
        &self.lifetime
    }

    pub fn bounds(&self) -> Option<&LifetimeBounds<'ast>> {
        self.bounds.get()
    }
}

impl<'ast> GenericParamData<'ast> for LifetimeParam<'ast> {
    fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| self.cx.get_span(*span))
    }
}

impl<'ast> From<&'ast LifetimeParam<'ast>> for GenericParamKind<'ast> {
    fn from(src: &'ast LifetimeParam<'ast>) -> Self {
        Self::Lifetime(src)
    }
}

/// <https://doc.rust-lang.org/stable/reference/trait-bounds.html>
#[repr(C)]
#[derive(Debug)]
pub struct LifetimeBounds<'ast> {
    cx: AstContext<'ast>,
    lifetimes: FfiSlice<'ast, &'ast Lifetime<'ast>>,
    span: FfiOption<SpanId>,
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeBounds<'ast> {
    #[must_use]
    pub fn new(cx: AstContext<'ast>, lifetimes: FfiSlice<'ast, &'ast Lifetime<'ast>>, span: FfiOption<SpanId>) -> Self {
        Self { cx, lifetimes, span }
    }
}

impl<'ast> LifetimeBounds<'ast> {
    pub fn lifetimes(&self) -> &[&Lifetime<'ast>] {
        self.lifetimes.get()
    }
    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| self.cx.get_span(*span))
    }
}

#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[derive(Debug, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub(crate) enum LifetimeKind {
    /// A lifetime with a label like `'ast`
    Label(SymbolId),
    /// The magic `'static` lifetime
    Static,
    /// The mysterious `'_` lifetime
    Infer,
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct Lifetime<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    span: FfiOption<SpanId>,
    kind: LifetimeKind,
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

impl<'ast> std::fmt::Debug for Lifetime<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lifetime")
            .field(
                "kind",
                &match self.kind {
                    LifetimeKind::Label(sym) => with_cx(self, |cx| cx.symbol_str(sym)),
                    LifetimeKind::Static => "'static".to_string(),
                    LifetimeKind::Infer => "'_".to_string(),
                },
            )
            .finish()
    }
}

impl<'ast> Lifetime<'ast> {
    /// Note that the `'static` lieftime is not a label and will therefore return `None`
    pub fn label(&self) -> Option<String> {
        match self.kind {
            LifetimeKind::Label(sym) => Some(with_cx(self, |cx| cx.symbol_str(sym))),
            _ => None,
        }
    }

    pub fn has_label(&self) -> bool {
        matches!(self.kind, LifetimeKind::Label(_))
    }

    pub fn is_static(&self) -> bool {
        matches!(self.kind, LifetimeKind::Static)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self.kind, LifetimeKind::Infer)
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.get_span(*span)))
    }
}
