#![doc = include_str!("../README.md")]
#![warn(clippy::index_refutable_slice)]

mod loader;
use linter_api::{ast::Crate, context::Context, LintPass};
use loader::ExternalLintCrateRegistry;

/// This struct is the interface used by lint drivers to pass transformed objects to
/// external lint passes.
pub struct Adapter<'ast> {
    #[allow(unused)]
    external_lint_crates: ExternalLintCrateRegistry<'ast>,
}

impl<'ast> Adapter<'ast> {
    #[must_use]
    pub fn new_from_env() -> Self {
        let external_lint_crates = ExternalLintCrateRegistry::new_from_env();
        Self { external_lint_crates }
    }

    pub fn process_krate(&mut self, cx: &'ast Context<'ast>, krate: &Crate<'ast>) {
        for item in krate.get_items() {
            self.external_lint_crates.check_item(cx, *item);
        }
    }
}
