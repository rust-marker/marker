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
        let name = item.name().unwrap_or_default();
        if name.starts_with("PRINT_TYPE") {
            cx.emit_lint(TEST_LINT, "Printing type for", item.ty().span().unwrap());
            eprintln!("{:#?}\n\n", item.ty());
        } else if name.starts_with("FIND_ITEM") {
            cx.emit_lint(TEST_LINT, "hey there is a static item here", item.span());
        }
    }
}
