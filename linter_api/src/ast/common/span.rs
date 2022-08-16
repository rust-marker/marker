use std::path::PathBuf;

use crate::{ast::item::ItemId, context::AstContext};

use super::{Applicability, BodyId, ItemPath, SpanId};

#[repr(C)]
#[doc(hidden)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum SpanSource<'ast> {
    File(&'ast PathBuf),
    Macro(&'ast ItemPath<'ast>),
}

#[derive(Clone)]
pub struct Span<'ast> {
    cx: &'ast AstContext<'ast>,
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

    /// Returns true, if both spans originate from the sane source. This can for
    /// instance be the same source file or macro expansion.
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

    /// Returns the code that this span references or `None` if the code in unavailable
    pub fn snippet(&self) -> Option<String> {
        self.cx.span_snipped(self)
    }

    /// Converts a span to a code snippet if available, otherwise returns the default.
    ///
    /// This is useful if you want to provide suggestions for your lint or more generally, if you
    /// want to convert a given `Span` to a `String`. To create suggestions consider using
    /// [`Span::snippet_with_applicability`] to ensure that the [`Applicability`] stays correct.
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

    /// Same as [`Span::snippet`], but it adapts the applicability level by following rules:
    ///
    /// - Applicability level `Unspecified` will never be changed.
    /// - If the span is inside a macro, change the applicability level to `MaybeIncorrect`.
    /// - If the default value is used and the applicability level is `MachineApplicable`, change it
    ///   to `HasPlaceholders`
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
    pub fn new(cx: &'ast AstContext<'ast>, source: SpanSource<'ast>, start: usize, end: usize) -> Self {
        Self { cx, source, start, end }
    }

    pub fn source(&self) -> SpanSource {
        self.source
    }
}

/// **Unstable**
///
/// This enum is used to requrest a `Span` instance from the driver context.
/// it is only an internal type to avoid mapping every `Span`, since they are
/// most often not needed.
#[repr(C)]
#[doc(hidden)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[cfg_attr(not(feature = "driver-api"), visibility::make(pub(crate)))]
enum SpanOwner {
    /// This requrests the `Span` belonging to the [`ItemId`].
    Item(ItemId),
    /// This requrests the `Span` belonging to the [`BodyId`].
    Body(BodyId),
    /// This requests the `Span` belonging to a driver generated [`SpanId`]
    SpecificSpan(SpanId),
}

impl From<ItemId> for SpanOwner {
    fn from(id: ItemId) -> Self {
        Self::Item(id)
    }
}

impl From<BodyId> for SpanOwner {
    fn from(id: BodyId) -> Self {
        Self::Body(id)
    }
}

impl From<SpanId> for SpanOwner {
    fn from(id: SpanId) -> Self {
        Self::SpecificSpan(id)
    }
}
