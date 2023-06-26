extern crate marker_api;

use marker_api::{ast::expr::ExprKind, context::AstContext};

marker_api::declare_lint!(
    DUMMY,
    Warn,
    "dummy",
);

pub fn check_expr<'ast>(cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr.id(), "X <-- starting with upper case", expr.span(), |_| {});
}

fn main() {}
