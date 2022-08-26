#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use linter_api::{
    ast::item::{ItemData, StaticItem},
    context::AstContext,
    lint::Lint,
    LintPass,
};

linter_api::interface::export_lint_pass!(TestLintPass);

linter_api::lint::declare_lint!(TEST_LINT, Warn, "test lint warning");

#[derive(Default)]
struct TestLintPass {}

impl<'ast> LintPass<'ast> for TestLintPass {
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        Box::new([TEST_LINT])
    }

    fn check_static_item(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
        cx.emit_lint(TEST_LINT, "hey there is a static item here", item.get_span());
    }
}
