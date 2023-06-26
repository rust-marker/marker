use marker_api::{ast::expr::ExprKind, context::AstContext, LintPass, LintPassInfo, LintPassInfoBuilder};

marker_api::declare_lint!(
    DIAG_MSG_CAPITAL_START,
    Warn,
    r#"
    ### What it does

    Diagnostic messages should start with lower case letter according to [rustc's dev guide].

    [rustc's dev guide]: https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-output-style-guide
    "#,
);

#[derive(Debug, Default)]
struct MarkerLintPass;

marker_api::export_lint_pass!(MarkerLintPass);

impl LintPass for MarkerLintPass {
    fn info(&self) -> LintPassInfo {
        LintPassInfoBuilder::new(Box::new([DIAG_MSG_CAPITAL_START])).build()
    }

    fn check_expr<'ast>(&mut self, cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
        cx.emit_lint(
            DIAG_MSG_CAPITAL_START,
            expr.id(),
            "X <-- starting with upper case",
            expr.span(),
            |_| {},
        );
    }
}
