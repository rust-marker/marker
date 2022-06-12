use crate::{
    ast::{Span, Symbol},
    lint::Lint,
};

/// This context will be passed to each [`super::LintPass`] call to enable the user
/// to emit lints and to retieve nodes by the given ids.
pub struct AstContext<'ast> {
    cx: &'ast dyn DriverContext<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstContext<'ast> {
    pub fn new(cx: &'ast dyn DriverContext<'ast>) -> Self {
        Self { cx }
    }
}

impl<'ast> AstContext<'ast> {
    pub fn emit_lint(&self, s: &str, lint: &Lint) {
        self.cx.emit_lint(s, lint);
    }

    pub fn emit_lint_span(&self, s: &str, lint: &Lint, sp: &dyn Span<'_>) {
        self.cx.emit_lint_span(s, lint, sp);
    }
}

/// This trait provides the actual implementation of [`AstContext`]. [`AstContext`] is just
/// a wrapper type to avoid writing `dyn` for every context and to prevent users from
/// implementing this trait.
pub trait DriverContext<'ast> {
    fn emit_lint(&self, s: &str, lint: &Lint);
    fn emit_lint_span(&self, s: &str, lint: &Lint, sp: &dyn Span<'_>);
}

/// This trait is used to create [`Symbol`]s and to turn them back into
/// strings. It might be better to have a single struct like rustc does
/// but I'm not sure how to properly implement this across crate bounderies.
/// Having this trait seams clean enough for now.
pub trait SymbolStore {
    fn to_sym(&mut self, string: &str) -> Symbol;

    fn sym_to_str(&self, sym: Symbol) -> &str;
}
