use linter_api::LintPass;

linter_api::interface::export_lint_pass!("linter_test", TestLintPass::new());

struct TestLintPass {}

impl TestLintPass {
    pub fn new() -> Self {
        Self {}
    }
}

impl LintPass for TestLintPass {
    fn test_call(&self, msg: &str) {
        println!("Message from test: {}", msg);
    }
}
