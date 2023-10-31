use std::marker::PhantomData;

use crate::{
    common::{ExpnId, MacroId, SpanId, SpanSrcId, SymbolId},
    context::with_cx,
    diagnostic::Applicability,
    ffi,
    private::Sealed,
};

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
/// In this example `ex3b!(2)` expands to `ex3a!(2, 3)` which in turn expands to
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
/// layers. First the root layer calling the `ex4b` macro, which calls the `ex4a`
/// macro, which inturn expands into the `4 + 4` expression.
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
    parent: ExpnId,
    call_site: SpanId,
    macro_id: MacroId,
}

impl<'ast> ExpnInfo<'ast> {
    /// This returns [`Some`] if this expansion comes from another expansion.
    #[must_use]
    pub fn parent(&self) -> Option<&ExpnInfo<'ast>> {
        with_cx(self, |cx| cx.span_expn_info(self.parent))
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
    pub fn new(parent: ExpnId, call_site: SpanId, macro_id: MacroId) -> Self {
        Self {
            _lifetime: PhantomData,
            parent,
            call_site,
            macro_id,
        }
    }
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
/// by setting the [`MacroReport`][crate::common::MacroReport] value during lint
/// creation. If your lint is targeting code from macro expansions, please
/// consider that users might not be able to influence the generated code. It's
/// also worth checking that all linted nodes originate from the same macro expansion.
/// Check out the documentation of [`ExpnInfo`].
#[repr(C)]
#[derive(Clone)]
pub struct Span<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    /// The source of this [`Span`]. The id space and position distribution is
    /// decided by the driver. To get the full source information it might be
    /// necessary to also pass the start and end position to the driver.
    source_id: SpanSrcId,
    /// This information could also be retrieved, by requesting the [`ExpnInfo`]
    /// of this span. However, from looking at Clippy and rustc lints, it looks
    /// like the main interest is, if this comes from a macro expansion, not from
    /// which one. Having this boolean flag will be sufficient to answer this simple
    /// question and will save on extra [`SpanSrcId`] mappings.
    from_expansion: bool,
    start: SpanPos,
    end: SpanPos,
}

impl<'ast> std::fmt::Debug for Span<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_pos(pos: Option<FilePos<'_>>) -> String {
            match pos {
                Some(pos) => format!("{}:{}", pos.line(), pos.column()),
                None => "[invalid]".to_string(),
            }
        }

        let src = self.source();
        let name = match src {
            SpanSource::File(file) => format!(
                "{}:{} - {}",
                file.file(),
                fmt_pos(file.try_to_file_pos(self.start)),
                fmt_pos(file.try_to_file_pos(self.end))
            ),
            SpanSource::Macro(expn) => format!("[Inside Macro] {:#?}", expn.call_site()),
            SpanSource::Buildin(_) => "[From Prelude]".to_string(),
        };
        f.debug_struct(&name).finish()
    }
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
    pub fn snippet_or<'a, 'b>(&self, default: &'a str) -> &'b str
    where
        'a: 'b,
        'ast: 'b,
    {
        self.snippet().unwrap_or(default)
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
    ///
    /// Here is an example, for constructing a string with two expressions `a` and `b`:
    ///
    /// ```rust,ignore
    /// let mut app = Applicability::MachineApplicable;
    /// let sugg = format!(
    ///     "{}..{}",
    ///     a.span().snippet_with_applicability("<expr-a>", &mut app),
    ///     b.span().snippet_with_applicability("<expr-b>", &mut app),
    /// );
    /// ```
    pub fn snippet_with_applicability<'a, 'b>(&self, placeholder: &'a str, applicability: &mut Applicability) -> &'b str
    where
        'a: 'b,
        'ast: 'b,
    {
        if *applicability != Applicability::Unspecified && self.is_from_expansion() {
            *applicability = Applicability::MaybeIncorrect;
        }
        self.snippet().unwrap_or_else(|| {
            if matches!(
                *applicability,
                Applicability::MachineApplicable | Applicability::MaybeIncorrect
            ) {
                *applicability = Applicability::HasPlaceholders;
            }
            placeholder
        })
    }

    /// Returns the length of the this [`Span`] in bytes.
    pub fn len(&self) -> usize {
        (self.end.0 - self.start.0)
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

    #[must_use]
    pub fn source(&self) -> SpanSource<'ast> {
        with_cx(self, |cx| cx.span_source(self))
    }
}

impl<'ast> HasSpan<'ast> for Span<'ast> {
    fn span(&self) -> &Span<'ast> {
        self
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Span<'ast> {
    #[must_use]
    pub fn new(source_id: SpanSrcId, from_expansion: bool, start: SpanPos, end: SpanPos) -> Self {
        Self {
            _lifetime: PhantomData,
            source_id,
            from_expansion,
            start,
            end,
        }
    }

    pub fn source_id(&self) -> SpanSrcId {
        self.source_id
    }
}

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum SpanSource<'ast> {
    File(&'ast FileInfo<'ast>),
    Macro(&'ast ExpnInfo<'ast>),
    Buildin(&'ast BuildinInfo<'ast>),
}

#[repr(C)]
#[derive(Debug)]
pub struct FileInfo<'ast> {
    file: ffi::FfiStr<'ast>,
    span_src: SpanSrcId,
}

impl<'ast> FileInfo<'ast> {
    pub fn file(&self) -> &str {
        self.file.get()
    }

    /// Tries to map the given [`SpanPos`] to a [`FilePos`]. It will return [`None`]
    /// if the given [`FilePos`] belongs to a different [`FileInfo`].
    pub fn try_to_file_pos(&self, span_pos: SpanPos) -> Option<FilePos> {
        with_cx(self, |cx| cx.span_pos_to_file_loc(self, span_pos))
    }

    /// Map the given [`SpanPos`] to a [`FilePos`]. This will panic, if the
    /// [`SpanPos`] doesn't belong to this [`FileInfo`]
    pub fn to_file_pos(&self, span_pos: SpanPos) -> FilePos {
        self.try_to_file_pos(span_pos).unwrap_or_else(|| {
            panic!(
                "the given span position `{span_pos:#?}` is out of range of the file `{}`",
                self.file.get()
            )
        })
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> FileInfo<'ast> {
    #[must_use]
    pub fn new(file: &'ast str, span_src: SpanSrcId) -> Self {
        Self {
            file: file.into(),
            span_src,
        }
    }

    pub fn span_src(&self) -> SpanSrcId {
        self.span_src
    }
}

/// A location inside a file.
///
/// [`SpanPos`] instances belonging to files can be mapped to [`FilePos`] with
/// the [`FileInfo`] from the [`SpanSource`] of the [`Span`] they belong to. See:
/// [`FileInfo::to_file_pos`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FilePos<'ast> {
    /// The lifetime is not needed right now, but I want to have it, to potentualy
    /// add more behavior to this struct.
    _lifetime: PhantomData<&'ast ()>,
    /// The 1-indexed line in bytes
    line: usize,
    /// The 1-indexed column in bytes
    column: usize,
}

impl<'ast> FilePos<'ast> {
    /// Returns the 1-indexed line location in bytes
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the 1-indexed column location in bytes
    pub fn column(&self) -> usize {
        self.column
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> FilePos<'ast> {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            _lifetime: PhantomData,
            line,
            column,
        }
    }
}

/// The [`Span`] belongs to something, which was generated by the Compiler. This
/// could be the imports from the prelude or the testing harness.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "driver-api", derive(Default))]
pub struct BuildinInfo<'ast> {
    /// The lifetime is not needed right now, but I want to have it, to potentualy
    /// add more behavior to this struct.
    _lifetime: PhantomData<&'ast ()>,
    /// `#[repr(C)]` requires a field, to make this a proper type. This is just
    /// the smallest one.
    _data: u8,
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
}

impl<'ast> HasSpan<'ast> for Ident<'ast> {
    fn span(&self) -> &Span<'ast> {
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

/// A trait for types, that provide a [`Span`]. It is implemented for all
/// AST nodes, [`Span`] itself, and for references to them as well.
///
/// This gives you the ability to invoke functions that take `impl HasSpan`
/// in many different ways. Just choose the one that fits your use case the best.
///
/// ```
/// # use marker_api::prelude::*;
///
/// fn takes_span<'ast>(span: impl HasSpan<'ast>) {
///     let span: &Span<'ast> = span.span();
///     // ...
/// }
///
/// fn visit_expr(expr: ExprKind<'_>) {
///     takes_span(expr);
///     takes_span(&expr);
///     takes_span(expr.span());
///     takes_span(&expr.span());
/// }
/// ```
pub trait HasSpan<'ast>: Sealed {
    /// This returns the [`Span`] of the implementing AST node.
    fn span(&self) -> &Span<'ast>;
}

macro_rules! impl_has_span_via_field {
    ($ty:ty) => {
        $crate::span::impl_has_span_via_field!($ty, span);
    };
    ($ty:ty, $($field_access:ident).*) => {
        impl<'ast> $crate::span::HasSpan<'ast> for $ty {
            fn span(&self) -> &$crate::span::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.span(self.$($field_access).*))
            }
        }
    }
}
pub(crate) use impl_has_span_via_field;

/// This macro implements the [`HasSpan`] trait for data types, that provide a
/// `span()` method.
macro_rules! impl_spanned_for {
    ($ty:ty) => {
        impl<'ast> $crate::span::HasSpan<'ast> for $ty {
            fn span(&self) -> &$crate::span::Span<'ast> {
                self.span()
            }
        }
    };
}
pub(crate) use impl_spanned_for;

impl<'ast, N: HasSpan<'ast>> HasSpan<'ast> for &N {
    fn span(&self) -> &Span<'ast> {
        (*self).span()
    }
}
