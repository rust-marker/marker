use linter_api::{lint::Lint, LintPass, ast::item::ItemType};

linter_api::interface::export_lint_pass!("linter_test", TestLintPass::new());

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

    fn check_item(&mut self, item: ItemType<'ast>) {
        dbg!(item);
    }
}
