#![doc = include_str!("../README.md")]
#![feature(let_chains)]

use marker_api::{
    ast::{expr::ExprKind, ty::SemTyKind},
    context::AstContext,
    LintPass, LintPassInfo, LintPassInfoBuilder,
};

marker_api::declare_lint!(
    /// ### What it does
    /// Diagnostic messages should start with lower case letter according to
    /// [rustc's dev guide].
    ///
    /// [rustc's dev guide]: <https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-output-style-guide>
    DIAG_MSG_UPPERCASE_START,
    Warn,
);

#[derive(Debug, Default)]
struct MarkerLintsLintPass;

marker_api::export_lint_pass!(MarkerLintsLintPass);

impl LintPass for MarkerLintsLintPass {
    fn info(&self) -> LintPassInfo {
        LintPassInfoBuilder::new(Box::new([DIAG_MSG_UPPERCASE_START])).build()
    }

    fn check_expr<'ast>(&mut self, cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
        if let ExprKind::Method(call) = expr
            && let SemTyKind::Ref(adt_ref) = call.receiver().ty()
            && let SemTyKind::Adt(adt) = adt_ref.inner_ty()
            && cx.resolve_ty_ids("marker_api::AstContext").contains(&adt.def_id())
            && call.method().ident().name() == "emit_lint"
            && let [_lint, _id, msg, ..] = call.args()
        {
            check_msg(cx, *msg);
        }
    }
}

fn check_msg<'ast>(cx: &AstContext<'ast>, msg_expr: ExprKind<'ast>) {
    if let ExprKind::StrLit(lit) = msg_expr
        && let Some(msg) = lit.str_value()
        && let Some(start) = msg.chars().next()
        && start.is_uppercase()
    {
        cx.emit_lint(
            DIAG_MSG_UPPERCASE_START,
            msg_expr,
            "this message starts with an uppercase character",
        );
    }
}
