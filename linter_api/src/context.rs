use crate::{
    ast::{Span, SpanOwner},
    lint::Lint,
};

/// This context will be passed to each [`super::LintPass`] call to enable the user
/// to emit lints and to retieve nodes by the given ids.
#[repr(C)]
pub struct AstContext<'ast> {
    driver: &'ast DriverCallbacks<'ast>,
}

impl<'ast> std::fmt::Debug for AstContext<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstContext").finish()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> AstContext<'ast> {
    pub fn new(driver: &'ast DriverCallbacks<'ast>) -> Self {
        Self { driver }
    }
}

impl<'ast> AstContext<'ast> {
    /// This function emits a lint at the current node with the given
    /// message and span.
    ///
    /// For rustc the text output will look roughly to this:
    /// ```txt
    /// error: ducks can't talk
    ///  --> $DIR/file.rs:17:5
    ///    |
    /// 17 |     println!("The duck said: 'Hello, World!'");
    ///    |
    /// ```
    pub fn emit_lint(&self, lint: &'static Lint, msg: &str, span: &Span<'ast>) {
        self.driver.call_emit_lint(lint, msg, span);
    }
}

impl<'ast> AstContext<'ast> {
    pub(crate) fn span_snipped(&self, span: &Span) -> Option<String> {
        self.driver.call_span_snippet(span)
    }

    pub(crate) fn get_span(&self, span_owner: &SpanOwner) -> &'ast Span<'ast> {
        self.driver.call_get_span(span_owner)
    }
}

/// This struct holds function pointers to driver implementations of required
/// functions. These can roughly be split into two categories:
///
/// 1. **Public utility**: These functions will be exposed to lint-crates via
///     an [`AstContext`] instance. Therefore, the function signature of these
///     has to be stable, or at least be stable for [`AstContext`].
/// 2. **Internal utility**: These functions are intended for internal usage
///     inside the API or the `linter_adapter` crate. Some nodes might also have
///     a reference to these callbacks to request additional information if
///     required. These are not part of the stable API and can therefore be changed.
///
/// Any changes to this struct will most likely require changes to the
/// `DriverContextWrapper` implementation in the `liner_adapter` crate. That
/// type provides a simple wrapper to avoid driver unrelated boilerplate code.
#[repr(C)]
#[doc(hidden)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct DriverCallbacks<'ast> {
    /// This is a pointer to the driver context, provided to each function as
    /// the first argument. This is an untyped pointer, since the driver is
    /// unknown to the api and adapter. The context has to be casted into the
    /// driver-specific type by the driver. A driver is always guaranteed to
    /// get its own context.
    pub driver_context: *const (),
    pub emit_lint: extern "C" fn(*const (), &'static Lint, &str, &Span<'ast>),
    pub get_span: extern "C" fn(*const (), &SpanOwner) -> &'ast Span<'ast>,
    pub span_snippet: extern "C" fn(*const (), &Span) -> Option<&'ast str>,
}

impl<'ast> DriverCallbacks<'ast> {
    fn call_emit_lint(&self, lint: &'static Lint, msg: &str, span: &Span<'ast>) {
        (self.emit_lint)(self.driver_context, lint, msg, span);
    }
    fn call_get_span(&self, span_owner: &SpanOwner) -> &'ast Span<'ast> {
        (self.get_span)(self.driver_context, span_owner)
    }
    fn call_span_snippet(&self, span: &Span) -> Option<String> {
        (self.span_snippet)(self.driver_context, span).map(str::to_string)
    }
}
