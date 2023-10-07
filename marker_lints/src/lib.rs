#![doc = include_str!("../README.md")]
#![feature(let_chains)]

mod diag_msg_uppercase_start;
mod not_using_has_span_trait;

use marker_api::{prelude::*, LintPass, LintPassInfo, LintPassInfoBuilder};

#[derive(Debug, Default)]
struct MarkerLintsLintPass;

marker_api::export_lint_pass!(MarkerLintsLintPass);

impl LintPass for MarkerLintsLintPass {
    fn info(&self) -> LintPassInfo {
        let lints = [
            diag_msg_uppercase_start::DIAG_MSG_UPPERCASE_START,
            not_using_has_span_trait::NOT_USING_HAS_SPAN_TRAIT,
        ];

        LintPassInfoBuilder::new(Box::new(lints)).build()
    }

    fn check_expr<'ast>(&mut self, cx: &MarkerContext<'ast>, expr: ExprKind<'ast>) {
        diag_msg_uppercase_start::check_expr(cx, expr);
    }

    fn check_item<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, item: ast::ItemKind<'ast>) {
        not_using_has_span_trait::check_item(cx, item);
    }
}
