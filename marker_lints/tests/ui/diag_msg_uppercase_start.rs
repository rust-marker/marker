extern crate marker_api;

use marker_api::prelude::*;

marker_api::declare_lint!{
    /// Dummy
    DUMMY,
    Warn,
}

pub fn accept_message<'ast>(cx: &MarkerContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr, "x <-- this is cool");
    cx.emit_lint(DUMMY, expr, "=^.^= <-- Interesting, but valid");
    cx.emit_lint(DUMMY, expr, "");

    let variable = "";
    cx.emit_lint(DUMMY, expr, variable);
}

pub fn warn_about_message<'ast>(cx: &MarkerContext<'ast>, expr: ExprKind<'ast>) {
    cx.emit_lint(DUMMY, expr, "X <-- starting with upper case");
    cx.emit_lint(DUMMY, expr, "Hey <-- starting with upper case");
}

fn main() {}
