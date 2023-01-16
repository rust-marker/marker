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
    lint::Lint,
    LintPass,
};

/// This struct is the interface used by lint drivers to pass transformed objects to
/// external lint passes.
pub struct Adapter<'ast> {
    external_lint_crates: LintCrateRegistry<'ast>,
}

impl<'ast> Adapter<'ast> {
    #[must_use]
    pub fn new_from_env() -> Self {
        let external_lint_crates = LintCrateRegistry::new_from_env();
        Self { external_lint_crates }
    }

    #[must_use]
    pub fn registered_lints(&self) -> Box<[&'static Lint]> {
        self.external_lint_crates.registered_lints()
    }

    pub fn process_krate(&mut self, cx: &'ast AstContext<'ast>, krate: &Crate<'ast>) {
        self.external_lint_crates.set_ast_context(cx);

        for item in krate.items() {
            self.external_lint_crates.check_item(cx, *item);
            self.process_item(cx, item);
        }
    }

    fn process_item(&mut self, cx: &'ast AstContext<'ast>, item: &ItemKind<'ast>) {
        match item {
            ItemKind::Mod(data) => {
                self.external_lint_crates.check_mod(cx, data);
                for item in data.items() {
                    self.process_item(cx, item);
                }
            },
            ItemKind::ExternCrate(data) => self.external_lint_crates.check_extern_crate(cx, data),
            ItemKind::Use(data) => self.external_lint_crates.check_use_decl(cx, data),
            ItemKind::Static(data) => self.external_lint_crates.check_static_item(cx, data),
            ItemKind::Const(data) => self.external_lint_crates.check_const_item(cx, data),
            // FIXME: Function-local items are not yet processed
            ItemKind::Fn(data) => {
                self.external_lint_crates.check_fn(cx, data);
                if let Some(id) = data.body() {
                    self.process_body(cx, id)
                }
            },
            ItemKind::Struct(data) => {
                self.external_lint_crates.check_struct(cx, data);
                for field in data.fields() {
                    self.external_lint_crates.check_field(cx, field);
                }
            },
            ItemKind::Enum(data) => {
                self.external_lint_crates.check_enum(cx, data);
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

    fn process_body(&mut self, cx: &'ast AstContext<'ast>, id: BodyId) {
        let body = cx.body(id);
        self.external_lint_crates.check_body(cx, body);
        let expr = body.expr();
        self.process_expr(cx, expr);
    }

    fn process_expr(&mut self, cx: &'ast AstContext<'ast>, expr: ExprKind<'ast>) {
        self.external_lint_crates.check_expr(cx, expr);
        match expr {
            ExprKind::Block(block) => {
                for stmt in block.stmts() {
                    self.external_lint_crates.check_stmt(cx, *stmt)
                }
            },
            _ => {},
        }
    }
}
