//! This module contains utilities to search a given part of the AST for a
//! different things.

use std::ops::ControlFlow;

use crate::visitor::{for_each_expr, Traverseable};

use marker_api::prelude::*;

/// Checks if the given node contains an early return, in the form of an
/// [`ReturnExpr`](marker_api::ast::expr::ReturnExpr) or
/// [`QuestionMarkExpr`](marker_api::ast::expr::QuestionMarkExpr).
///
/// This function is useful, for lints which suggest moving code snippets into
/// a closure or different function. Return statements might prevent the suggested
/// refactoring.
pub fn contains_return<'ast>(cx: &'ast AstContext<'ast>, node: impl Traverseable<'ast, bool>) -> bool {
    for_each_expr(cx, node, |expr| {
        if matches!(expr, ExprKind::Return(_) | ExprKind::QuestionMark(_)) {
            ControlFlow::Break(true)
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_some()
}
