extern crate marker_api;

use marker_api::{ast::expr::ExprKind, context::AstContext};

marker_api::declare_lint!(
    /// Dummy
    DUMMY,
    Warn,
);

pub fn accept_message<'ast>(cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr.id(), "x <-- this is cool", expr.span(), |_| {});
    cx.emit_lint(DUMMY, expr.id(), "=^.^= <-- Interesting, but valid", expr.span(), |_| {});
    cx.emit_lint(DUMMY, expr.id(), "", expr.span(), |_| {});
    
    let variable = "";
    cx.emit_lint(DUMMY, expr.id(), variable, expr.span(), |_| {});
}

pub fn warn_about_message<'ast>(cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr.id(), "X <-- starting with upper case", expr.span(), |_| {});
    cx.emit_lint(DUMMY, expr.id(), "Hey <-- starting with upper case", expr.span(), |_| {});
}

fn main() {}
