#![doc = include_str!("../README.md")]
#![feature(lint_reasons)]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

pub mod context;
mod loader;

use loader::LintCrateRegistry;
use marker_api::{
    ast::{expr::ExprKind, item::ItemKind, BodyId, Crate},
    context::AstContext,
    LintPass, LintPassInfo,
};

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
            self.process_item(cx, item);
        }
    }

    fn process_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &ItemKind<'ast>) {
        self.external_lint_crates.check_item(cx, *item);
        match item {
            ItemKind::Mod(data) => {
                for item in data.items() {
                    self.process_item(cx, item);
                }
            },
            // FIXME: Function-local items are not yet processed
            ItemKind::Fn(data) => {
                if let Some(id) = data.body_id() {
                    self.process_body(cx, id);
                }
            },
            ItemKind::Struct(data) => {
                for field in data.fields() {
                    self.external_lint_crates.check_field(cx, field);
                }
            },
            ItemKind::Enum(data) => {
                for variant in data.variants() {
                    self.external_lint_crates.check_variant(cx, variant);
                    for field in variant.fields() {
                        self.external_lint_crates.check_field(cx, field);
                    }
                }
            },
            _ => {},
        }
    }

    fn process_body<'ast>(&mut self, cx: &'ast AstContext<'ast>, id: BodyId) {
        let body = cx.body(id);
        self.external_lint_crates.check_body(cx, body);
        let expr = body.expr();
        self.process_expr(cx, expr);
    }

    fn process_expr<'ast>(&mut self, cx: &'ast AstContext<'ast>, expr: ExprKind<'ast>) {
        self.external_lint_crates.check_expr(cx, expr);
        #[expect(clippy::single_match)]
        match expr {
            ExprKind::Block(block) => {
                for stmt in block.stmts() {
                    self.external_lint_crates.check_stmt(cx, *stmt);
                }
                if let Some(expr) = block.expr() {
                    self.process_expr(cx, expr);
                }
            },
            _ => {},
        }
    }
}
