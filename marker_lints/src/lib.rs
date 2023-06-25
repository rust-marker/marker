use marker_api::{lint::Lint, LintPass};

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
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        Box::new([DIAG_MSG_CAPITAL_START])
    }
}
