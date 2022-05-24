#![doc = include_str!("../README.md")]

use linter_api::{ast::item::StaticItem, context::AstContext, lint::Lint, LintPass};

linter_api::interface::export_lint_pass!("linter", TestLintPass::new());

linter_api::lint::declare_lint!(TEST_LINT, Allow, "");

struct TestLintPass {}

impl TestLintPass {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'ast> LintPass<'ast> for TestLintPass {
    fn registered_lints(&self) -> Vec<&'static Lint> {
        vec![TEST_LINT]
    }

    fn check_static_item(&mut self, _cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
        dbg!(item);
    }
}
