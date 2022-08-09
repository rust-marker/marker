use crate::{ast::Span, lint::Lint};

/// This context will be passed to each [`super::LintPass`] call to enable the user
/// to emit lints and to retieve nodes by the given ids.
pub struct AstContext<'ast> {
    driver: &'ast DriverCallbacks<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstContext<'ast> {
    pub fn new(driver: &'ast DriverCallbacks<'ast>) -> Self {
        Self { driver }
    }
}

impl<'ast> AstContext<'ast> {
    pub fn emit_lint(&self, msg: &str, lint: &'static Lint) {
        self.driver.call_emit_lint_without_span(lint, msg)
    }

    pub fn emit_lint_span(&self, msg: &str, lint: &'static Lint, span: &dyn Span<'ast>) {
        self.driver.call_emit_lint(lint, msg, span)
    }
}

#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct DriverCallbacks<'ast> {
    /// This is a pointer to the driver context, provided to each function as
    /// the first argument. This is an untyped pointer, since the driver is
    /// unknown to the api and adapter. The context has to be casted into the
    /// driver-specific type by the driver. A driver is always guaranteed to
    /// get its own context.
    driver_context: *const (),
    pub emit_lint: extern "C" fn(*const (), &'static Lint, &str, &dyn Span<'ast>),
    pub emit_lint_without_span: extern "C" fn(*const (), &'static Lint, &str),
}

impl<'ast> DriverCallbacks<'ast> {
    pub fn new(driver_context: *const ()) -> Self {
        DriverCallbacks {
            driver_context,
            emit_lint: dummy_emit_lint,
            emit_lint_without_span: dummy_emit_lint_without_span,
        }
    }
}

extern "C" fn dummy_emit_lint<'ast>(_data: *const (), _lint: &'static Lint, _msg: &str, _span: &dyn Span<'ast>) {
    unimplemented!()
}
extern "C" fn dummy_emit_lint_without_span(_data: *const (), _lint: &'static Lint, _msg: &str) {
    unimplemented!()
}

impl<'ast> DriverCallbacks<'ast> {
    fn call_emit_lint(&self, lint: &'static Lint, msg: &str, span: &dyn Span<'ast>) {
        (self.emit_lint)(self.driver_context, lint, msg, span)
    }
    fn call_emit_lint_without_span(&self, lint: &'static Lint, msg: &str) {
        (self.emit_lint_without_span)(self.driver_context, lint, msg)
    }
}
