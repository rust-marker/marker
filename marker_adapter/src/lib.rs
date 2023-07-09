#![doc = include_str!("../README.md")]
#![feature(lint_reasons)]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

pub mod context;
mod loader;

use std::ops::ControlFlow;

use loader::LintCrateRegistry;
use marker_api::{
    ast::{
        expr::ExprKind,
        item::{Body, EnumVariant, Field, ItemKind},
        stmt::StmtKind,
        Crate,
    },
    context::AstContext,
    LintPass, LintPassInfo,
};
use marker_utils::visitor::{self, Visitor};

/// This struct is the interface used by lint drivers to pass transformed objects to
/// external lint passes.
pub struct Adapter {
    external_lint_crates: LintCrateRegistry,
}

impl Adapter {
    #[must_use]
    pub fn new_from_env() -> Self {
        let external_lint_crates = LintCrateRegistry::new_from_env();
        Self { external_lint_crates }
    }

    #[must_use]
    pub fn registered_lints(&self) -> Vec<LintPassInfo> {
        self.external_lint_crates.collect_lint_pass_info()
    }

    pub fn process_krate<'ast>(&mut self, cx: &'ast AstContext<'ast>, krate: &Crate<'ast>) {
        self.external_lint_crates.set_ast_context(cx);

        for item in krate.items() {
            visitor::traverse_item::<()>(cx, self, *item);
        }
    }
}

impl Visitor<()> for Adapter {
    fn visit_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: ItemKind<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_item(cx, item);
        ControlFlow::Continue(())
    }

    fn visit_field<'ast>(&mut self, cx: &'ast AstContext<'ast>, field: &'ast Field<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_field(cx, field);
        ControlFlow::Continue(())
    }

    fn visit_variant<'ast>(&mut self, cx: &'ast AstContext<'ast>, variant: &'ast EnumVariant<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_variant(cx, variant);
        ControlFlow::Continue(())
    }

    fn visit_body<'ast>(&mut self, cx: &'ast AstContext<'ast>, body: &'ast Body<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_body(cx, body);
        ControlFlow::Continue(())
    }

    fn visit_stmt<'ast>(&mut self, cx: &'ast AstContext<'ast>, stmt: StmtKind<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_stmt(cx, stmt);
        ControlFlow::Continue(())
    }

    fn visit_expr<'ast>(&mut self, cx: &'ast AstContext<'ast>, expr: ExprKind<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_expr(cx, expr);
        ControlFlow::Continue(())
    }
}
