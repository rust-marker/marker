use std::marker::PhantomData;

use crate::{context::with_cx, diagnostic::Applicability, ffi};

use super::{MacroId, SpanId, SpanSrcId, SymbolId};

/// A byte position used for the start and end position of [`Span`]s.
///
/// This position can map to a source file or a virtual space, when it comes
/// from the expansion of a macro. It's expected that a [`SpanPos`] always
/// points to the start of a character. Indexing to the middle of a multi byte
/// character can result in panics.
///
/// The start [`SpanPos`] doesn't have to start at zero, the order can be decided
/// by the driver. A [`Span`] should always use [`SpanPos`] from the same span source.
///
/// **Stability notice**:
/// * The position may not be stable between different sessions.
/// * [`SpanPos`] should never be stored by lint crates, as drivers might change [`SpanPos`] between
///   different `check_*` function calls.
/// * The layout and size of this type might change. The type will continue to provide the current
///   trait implementations.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SpanPos(
    /// Rustc only uses u32, therefore it should be safe to do the same. This
    /// allows crates to have a total span size of ~4 GB (with expanded macros).
    /// That sounds reasonable :D
    u32,
);

#[cfg(feature = "driver-api")]
impl SpanPos {
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> u32 {
        self.0
    }
}

/// Information about a specific expansion.
///
/// [`Span`]s in Rust are structured in layers. The root layer is the source code
/// written in a source file. Macros can be expanded to AST nodes with [`Span`]s.
/// These are then added as a new layer, on top of the root. This struct provides
/// the information about one expansion layer.
///
/// ### Simple Macro Rules
///
/// ```
/// macro_rules! ex1 {
///     () => {
///         1 + 1
///     };
/// }
///
/// ex1!();
/// ```
///
/// In this example `ex1!()` expands into the expression `1 + 1`. The [`Span`]
/// of the binary expression and numbers will all be from an expansion. Snipping the
/// [`Span`] of the binary expression would return `1 + 1` from inside the macro rules.
///
/// ### Macro Rules with Parameters
///
/// ```
/// macro_rules! ex2 {
///     ($a:literal, $b:literal) => {
///         $a + $b
///     };
/// }
///
/// ex2!(1, 2);
/// ```
///
/// In this example `ex2!(1, 2)` expands into the expression `1 + 2`. The [`Span`]
/// of the binary expression is marked to come from the expansion of a macro.
/// The `1` and `2` literals are marked as coming from the root layer, since
/// they're actually written in the source file.
///
/// ### Macros Invoking Macros
///
/// ```
/// macro_rules! ex3a {
///     ($a:literal, $b:literal) => {
///         $a + $b
///     };
/// }
/// macro_rules! ex3b {
///     ($a:literal) => {
///         ex3a!($a, 3)
///     };
/// }
///
/// ex3b!(2);
/// ```
///
/// In this example `ex3b!(2)` expands to `ex3a(2, 3)` which in turn expands to
/// `2 + 3`. This expansion has three layers, first the root, which contains the
/// `ex3b!(2)` call. The next layer is the `ex3a!(2, 3)` call. The binary expression
/// comes from the third layer, the number 3 from the second, and the number 2 from
/// the root layer, since this one was actually written by the user.
///
/// ### Macros Creating Macros
///
/// ```
/// macro_rules! ex4a {
///     () => {
///         macro_rules! ex4b {
///             () => {
///                 4 + 4
///             };
///         }
///     };
/// }
///
/// ex4a!();
/// ex4b!();
/// ```
///
/// This example expands `ex4a` into a new `ex4b` macro, which in turn expands
/// into a `4 + 4` expression. The [`Span`] of the binary expression has three
/// layers. First the root layer calling the `ex4b` macro, this would then
/// expand to the `4 + 4` expression, which actually comes from the `ex4a` expansion.
///
/// ### Proc Macros
///
/// Proc macros and some built-in macros are different from `macro_rules!` macros as
/// they are opaque to the driver. It's just known that some tokens are provided as an
/// input and somehow expanded. The [`Span`]s of the expanded tokens are marked as
/// coming from an expansion by default. However, macro crates can sometimes override
/// this with some trickery. (Please use this forbidden knowledge carefully.)
#[repr(C)]
#[derive(Debug)]
pub struct ExpnInfo<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    parent: SpanSrcId,
    call_site: SpanId,
    macro_id: MacroId,
}

impl<'ast> ExpnInfo<'ast> {
    #[must_use]
    pub fn parent(&self) -> Option<&ExpnInfo<'ast>> {
        // TODO(xFrednet): Request info from Driver
        todo!()
    }

    /// The [`Span`] that invoked the macro, that this expansion belongs to.
    #[must_use]
    pub fn call_site(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.call_site))
    }

    pub fn macro_id(&self) -> MacroId {
        self.macro_id
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ExpnInfo<'ast> {
    #[must_use]
    pub fn new(parent: SpanSrcId, call_site: SpanId, macro_id: MacroId) -> Self {
        Self {
            _lifetime: PhantomData,
            parent,
            call_site,
            macro_id,
        }
    }
}

// FIXME(xFrednet): This enum is "limited" to say it lightly, it should contain
// the more information about macros and their expansion etc. This covers the
// basic use case of checking if a span comes from a macro or a file. The rest
// will come in due time. Luckily it's not a public enum right now.
//
// See: rust-marker/marker#175
#[repr(C)]
#[allow(clippy::exhaustive_enums)]
#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum SpanSource<'ast> {
    /// The span comes from a file
    File(ffi::FfiStr<'ast>),
    /// The span comes from a macro.
    Macro(SpanSrcId),
    /// The span belongs to a file, but is the result of desugaring, they should
    /// be handled like normal files. This is variant mostly important for the driver.
    // FIXME(xFrednet): All desugars are usually resugared by Marker, the driver should
    // pass the span of the original sugared expression to the API.
    Sugar(ffi::FfiStr<'ast>, SpanSrcId),
}

/// A region of code, used for snipping, lint emission, and the retrieval of
/// context information.
///
/// [`Span`]s provide context information, like the file location or macro expansion
/// that created this span. [`SpanPos`] values from different sources or files should
/// not be mixed. Check out the documentation of [`SpanPos`] for more information.
///
/// [`Span`]s don't provide any way to map back to the AST nodes, that they
/// belonged to. If you require this information, consider passing the nodes
/// instead or alongside the [`Span`].
///
/// [`Span`]s with invalid positions in the `start` or `end` value can cause panics
/// in the driver. Please handle them with care, and also consider that UTF-8 allows
/// multiple bytes per character. Instances provided by the API or driver directly,
/// are always valid.
///
/// Handling macros during linting can be difficult, generally it's advised to
/// abort, if the code originates from a macro. The API provides an automatic way
/// by setting the [`MacroReport`][crate::lint::MacroReport] value during lint
/// creation. If your lint is targeting code from macro expansions, please
/// consider that users might not be able to influence the generated code. It's
/// also worth checking that all linted nodes originate from the same macro expansion.
/// Check out the documentation of [`ExpnInfo`].
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Span<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    /// The source of this [`Span`]. The id space and position distribution is
    /// decided by the driver. To get the full source information it might be
    /// necessary to also pass the start and end position to the driver.
    source: SpanSrcId,
    /// This information could also be retrieved, by requesting the [`ExpnInfo`]
    /// of this span. However, from looking at Clippy and rustc lints, it looks
    /// like the main interested is, if this comes from a macro expansion, not from
    /// which one. Having this boolean flag will be sufficient to answer this simple
    /// question and will save on extra [`SpanSrcId`] mappings.
    from_expansion: bool,
    start: SpanPos,
    end: SpanPos,
}

impl<'ast> Span<'ast> {
    /// Returns `true`, if this [`Span`] comes from a macro expansion.
    pub fn is_from_expansion(&self) -> bool {
        self.from_expansion
    }

    /// Returns the code snippet that this [`Span`] refers to or [`None`] if the
    /// snippet is unavailable.
    ///
    /// ```ignore
    /// let variable = 15_000;
    /// //             ^^^^^^
    /// //             lit_span
    ///
    /// lit_span.snippet(); // -> Some("15_000")
    /// ```
    ///
    /// There are several reasons, why a snippet might be unavailable. Also
    /// depend on the used driver. You can also checkout the other snippet
    /// methods to better deal with these cases:
    /// * [`snippet_or`](Self::snippet_or)
    /// * [`snippet_with_applicability`](Self::snippet_with_applicability)
    #[must_use]
    pub fn snippet(&self) -> Option<&'ast str> {
        with_cx(self, |cx| cx.span_snipped(self))
    }

    /// Returns the code snippet that this [`Span`] refers to or the given default
    /// if the snippet is unavailable.
    ///
    /// For placeholders, it's recommended to use angle brackets with information
    /// should be filled out. For example, if you want to snip an expression, you
    /// should use `<expr>` as the default value.
    ///
    /// If you're planning to use this snippet in a suggestion, consider using
    /// [`snippet_with_applicability`](Self::snippet_with_applicability) instead.
    pub fn snippet_or(&self, default: &str) -> String {
        self.snippet().unwrap_or(default).to_string()
    }

    /// Adjusts the given [`Applicability`] according to the context and returns the
    /// code snippet that this [`Span`] refers to or the given default if the
    /// snippet is unavailable.
    ///
    /// For the placeholder, it's recommended to use angle brackets with information
    /// should be filled out. A placeholder for an expression should look like
    /// this: `<expr>`
    ///
    /// The applicability will never be upgraded by this method. When you draft
    /// suggestions, you'll generally start with the highest [`Applicability`]
    /// your suggestion should have, and then use it with this snippet function
    /// to adjust it accordingly. The applicability is then used to submit the
    /// suggestion to the driver.
    pub fn snippet_with_applicability(&self, placeholder: &str, applicability: &mut Applicability) -> String {
        if *applicability != Applicability::Unspecified && self.is_from_expansion() {
            *applicability = Applicability::MaybeIncorrect;
        }
        self.snippet()
            .unwrap_or_else(|| {
                if matches!(
                    *applicability,
                    Applicability::MachineApplicable | Applicability::MaybeIncorrect
                ) {
                    *applicability = Applicability::HasPlaceholders;
                }
                placeholder
            })
            .to_string()
    }

    /// Returns the length of the this [`Span`] in bytes.
    pub fn len(&self) -> usize {
        (self.start.0 - self.end.0)
            .try_into()
            .expect("Marker is not compiled for usize::BITs < 32")
    }

    /// Returns `true` if the span has a length of 0. This means that no bytes are
    /// inside the span.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the start position of this [`Span`].
    pub fn start(&self) -> SpanPos {
        self.start
    }

    /// Sets the start position of this [`Span`].
    pub fn set_start(&mut self, start: SpanPos) {
        assert!(
            start.0 <= self.end.0,
            "the start position should always be <= of the end position"
        );
        self.start = start;
    }

    /// Returns a new [`Span`] with the given start position.
    #[must_use]
    pub fn with_start(&self, start: SpanPos) -> Span<'ast> {
        let mut new_span = self.clone();
        new_span.set_start(start);
        new_span
    }

    /// Returns the end position of this [`Span`].
    pub fn end(&self) -> SpanPos {
        self.end
    }

    /// Sets the end position of this [`Span`].
    pub fn set_end(&mut self, end: SpanPos) {
        assert!(
            self.start.0 <= end.0,
            "the start position should always be >= of the end position"
        );
        self.end = end;
    }

    /// Returns a new [`Span`] with the given end position.
    #[must_use]
    pub fn with_end(&self, end: SpanPos) -> Span<'ast> {
        let mut new_span = self.clone();
        new_span.set_end(end);
        new_span
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Span<'ast> {
    #[must_use]
    pub fn new(source: SpanSrcId, from_expansion: bool, start: SpanPos, end: SpanPos) -> Self {
        Self {
            _lifetime: PhantomData,
            source,
            from_expansion,
            start,
            end,
        }
    }

    pub fn source_id(&self) -> SpanSrcId {
        self.source
    }
}

#[repr(C)]
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

impl<'ast> std::fmt::Display for Ident<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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
