extern crate marker_api;

use marker_api::{ast::expr::ExprKind, context::AstContext};

marker_api::declare_lint!(
    /// Dummy
    DUMMY,
    Warn,
);

pub fn accept_message<'ast>(cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr, "x <-- this is cool", |_| {});
    cx.emit_lint(DUMMY, expr, "=^.^= <-- Interesting, but valid", |_| {});
    cx.emit_lint(DUMMY, expr, "", |_| {});
    
    let variable = "";
    cx.emit_lint(DUMMY, expr, variable, |_| {});
}

pub fn warn_about_message<'ast>(cx: &AstContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr, "X <-- starting with upper case", |_| {});
    cx.emit_lint(DUMMY, expr, "Hey <-- starting with upper case", |_| {});
}

fn main() {}
