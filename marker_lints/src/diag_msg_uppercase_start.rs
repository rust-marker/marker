use marker_api::prelude::*;

marker_api::declare_lint! {
    /// ### What it does
    /// Diagnostic messages should start with lower case letter according to
    /// [rustc's dev guide].
    ///
    /// [rustc's dev guide]: <https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-output-style-guide>
    DIAG_MSG_UPPERCASE_START,
    Warn,
}

pub(crate) fn check_expr<'ast>(cx: &MarkerContext<'ast>, expr: ExprKind<'ast>) {
    if let ExprKind::Method(call) = expr
        && let sem::TyKind::Ref(adt_ref) = call.receiver().ty()
        && let sem::TyKind::Adt(adt) = adt_ref.inner_ty()
        && cx.resolve_ty_ids("marker_api::MarkerContext").contains(&adt.def_id())
        && call.method().ident().name() == "emit_lint"
        && let [_lint, _id, msg, ..] = call.args()
    {
        check_msg(cx, *msg);
    }
}

fn check_msg<'ast>(cx: &MarkerContext<'ast>, msg_expr: ExprKind<'ast>) {
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
