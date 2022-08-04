#![doc = include_str!("../README.md")]
#![feature(lint_reasons)]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]

mod loader;
use linter_api::{
    ast::{item::ItemType, Crate},
    context::AstContext,
    lint::Lint,
    LintPass,
};
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

    pub fn process_krate(&mut self, cx: &'ast AstContext<'ast>, krate: &Crate<'ast>) {
        for item in krate.get_items() {
            self.external_lint_crates.check_item(cx, *item);
        }

        for item in krate.get_items() {
            match item {
                ItemType::Mod(data) => self.external_lint_crates.check_mod(cx, *data),
                ItemType::ExternCrate(data) => self.external_lint_crates.check_extern_crate(cx, *data),
                ItemType::UseDecl(data) => self.external_lint_crates.check_use_decl(cx, *data),
                ItemType::Static(data) => self.external_lint_crates.check_static_item(cx, data),
                _ => {},
            }
        }
    }

    pub fn registered_lints(&self) -> Box<[&'static Lint]> {
        self.external_lint_crates.registered_lints()
    }
}
