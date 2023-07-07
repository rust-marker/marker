use std::{marker::PhantomData, path::PathBuf};

use crate::{context::with_cx, diagnostic::Applicability};

use super::{AstPath, ItemId, SpanId, SymbolId};

#[repr(C)]
#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[allow(clippy::exhaustive_enums)]
enum SpanSource<'ast> {
    File(&'ast PathBuf),
    Macro(&'ast AstPath<'ast>),
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Span<'ast> {
    source: SpanSource<'ast>,
    /// The start marks the first byte in the [`SpanSource`] that is included in this
    /// span. The span continues until the end position.
    start: usize,
    end: usize,
}

impl<'ast> Span<'ast> {
    pub fn is_from_file(&self) -> bool {
        matches!(self.source, SpanSource::File(..))
    }

    pub fn is_from_macro(&self) -> bool {
        matches!(self.source, SpanSource::Macro(..))
    }

    /// Returns `true` if the span has a length of 0. This means that no bytes are
    /// inside the span.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns true, if both spans originate from the same source. For example, this can be the
    /// same source file or macro expansion.
    pub fn is_same_source(&self, other: &Span<'ast>) -> bool {
        self.source == other.source
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    /// Returns the code that this span references or [`None`] if the code is unavailable.
    pub fn snippet(&self) -> Option<String> {
        with_cx(self, |cx| cx.span_snipped(self))
    }

    /// Converts a span to a code snippet if available, otherwise returns the default.
    ///
    /// This is useful if you want to provide suggestions for your lint or more generally, if you
    /// want to convert a given [`Span`] to a [`String`]. To create suggestions consider using
    /// [`snippet_with_applicability()`](`Self::snippet_with_applicability`) to ensure that the
    /// [`Applicability`] stays correct.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Given two spans one for `value` and one for the `init` expression.
    /// let value = Vec::new();
    /// //  ^^^^^   ^^^^^^^^^^
    /// //  span1   span2
    ///
    /// // The snipped call would return the corresponding code snippets
    /// span1.snippet_or_else("..") // -> "value"
    /// span2.snippet_or_else("..") // -> "Vec::new()"
    /// ```
    pub fn snippet_or_else(&self, default: &str) -> String {
        self.snippet().unwrap_or_else(|| default.to_string())
    }

    /// Same as [`snippet()`](`Self::snippet`), but adapts the applicability level by following
    /// rules:
    ///
    /// - Applicability level [`Unspecified`](`Applicability::Unspecified`) will never be changed.
    /// - If the span is inside a macro, change the applicability level to
    ///   [`MaybeIncorrect`](`Applicability::MaybeIncorrect`).
    /// - If the default value is used and the applicability level is
    ///   [`MachineApplicable`](`Applicability::MachineApplicable`), change it to
    ///   [`HasPlaceholders`](`Applicability::HasPlaceholders`)
    pub fn snippet_with_applicability(&self, default: &str, applicability: &mut Applicability) -> String {
        if *applicability != Applicability::Unspecified && self.is_from_macro() {
            *applicability = Applicability::MaybeIncorrect;
        }
        self.snippet().unwrap_or_else(|| {
            if *applicability == Applicability::MachineApplicable {
                *applicability = Applicability::HasPlaceholders;
            }
            default.to_string()
        })
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Span<'ast> {
    pub fn new(source: SpanSource<'ast>, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub fn source(&self) -> SpanSource<'ast> {
        self.source
    }
}

/// **Unstable**
///
/// This enum is used to request a [`Span`] instance from the driver context.
/// it is only an internal type to avoid mapping every [`Span`], since they are
/// most often not needed.
#[repr(C)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) enum SpanOwner {
    /// This requests the [`Span`] belonging to the [`ItemId`].
    Item(ItemId),
    /// This requests the [`Span`] belonging to a driver generated [`SpanId`]
    SpecificSpan(SpanId),
}

impl From<ItemId> for SpanOwner {
    fn from(id: ItemId) -> Self {
        Self::Item(id)
    }
}

impl From<SpanId> for SpanOwner {
    fn from(id: SpanId) -> Self {
        Self::SpecificSpan(id)
    }
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub struct Ident<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    sym: SymbolId,
    span: SpanId,
}

impl<'ast> Ident<'ast> {
    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.sym))
    }

    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Ident<'ast> {
    pub fn new(sym: SymbolId, span: SpanId) -> Self {
        Self {
            _lifetime: PhantomData,
            sym,
            span,
        }
    }
}

impl<'ast> std::fmt::Debug for Ident<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ident")
            .field("name", &self.name())
            .field("span", &self.span())
            .finish()
    }
}

macro_rules! impl_ident_eq_for {
    ($ty:ty) => {
        impl<'ast> PartialEq<$ty> for Ident<'ast> {
            fn eq(&self, other: &$ty) -> bool {
                self.name().eq(other)
            }
        }
        impl<'ast> PartialEq<Ident<'ast>> for $ty {
            fn eq(&self, other: &Ident<'ast>) -> bool {
                other.name().eq(self)
            }
        }
    };
    ($($ty:ty),+) => {
        $(
            impl_ident_eq_for!($ty);
        )+
    };
}

use impl_ident_eq_for;

impl_ident_eq_for!(
    str,
    String,
    std::ffi::OsStr,
    std::ffi::OsString,
    std::borrow::Cow<'_, str>
);
