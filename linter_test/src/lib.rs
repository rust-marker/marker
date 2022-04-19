use linter_api::{ast::item::ItemType, context::Context, lint::Lint, LintPass};

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

    fn check_item(&mut self, _cx: &Context<'ast>, item: ItemType<'ast>) {
        dbg!(item);
    }
}
