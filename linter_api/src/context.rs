use crate::ast::Symbol;

/// This context will be passed to each [`super::LintPass`] call to enable the user
/// to emit lints and to retieve nodes by the given ids.
///
/// For now it's just a dummy trait.
pub trait LintPassContext<'ast> {}

/// This trait is used to create [`Symbol`]s and to turn them back into
/// strings. It might be better to have a single struct like rustc does
/// but I'm not sure how to properly implement this across crate bounderies.
/// Having this trait seams clean enough for now.
pub trait SymbolStore {
    fn to_sym(&mut self, string: &str) -> Symbol;

    fn sym_to_str(&self, sym: Symbol) -> &str;
}
