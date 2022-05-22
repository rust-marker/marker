#![doc = include_str!("../README.md")]

use linter_api::{
    ast::item::{ExternCrateItem, ItemType},
    context::AstContext,
    lint::Lint,
    LintPass,
};

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

    fn check_item(&mut self, _cx: &AstContext<'ast>, item: ItemType<'ast>) {
        if let ItemType::Function(func) = item {
            dbg!(func);
        }
    }

    fn check_extern_crate(&mut self, _cx: &'ast AstContext<'ast>, extern_crate_item: &'ast dyn ExternCrateItem<'ast>) {
        dbg!(extern_crate_item);
    }
}
