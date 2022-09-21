use crate::context::AstContext;

use super::{Span, SpanId, SymbolId};

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) enum LifetimeKind {
    /// A lifetime with a label like `'ast`
    Named(SymbolId),
    /// The magic `'static` lifetime
    Static,
    /// The mysterious `'_` lifetime
    Wildcard,
}

/// This struct represents a syntactic lifetime like `&'a ()`
#[repr(C)]
#[derive(Debug)]
pub struct Lifetime<'ast> {
    // FIXME: It might be helpful to add a `LifetimeId` to provide additional
    // information. However, this information might not be available from every
    // driver.
    cx: &'ast AstContext<'ast>,
    kind: LifetimeKind,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> Lifetime<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, kind: LifetimeKind, span: SpanId) -> Self {
        Self { cx, kind, span }
    }
}

impl<'ast> Lifetime<'ast> {
    /// Note that the `'static` lieftime is not a label and will therefore return `None`
    pub fn label(&self) -> Option<String> {
        match self.kind {
            LifetimeKind::Named(sym) => Some(self.cx.symbol_str(sym)),
            _ => None,
        }
    }

    /// Returns true, if this is a labeled lifetime like `'a`
    pub fn is_labeled(&self) -> bool {
        matches!(self.kind, LifetimeKind::Named(_))
    }

    /// Returns `true`, if this is the `'static` lifetime
    pub fn is_static(&self) -> bool {
        matches!(self.kind, LifetimeKind::Static)
    }

    /// Returns `true`, if this is a wildcard lifetime like `'_`
    pub fn is_wildcard(&self) -> bool {
        matches!(self.kind, LifetimeKind::Wildcard)
    }

    pub fn span(&self) -> &Span<'ast> {
        self.cx.get_span(&self.span.into())
    }
}
