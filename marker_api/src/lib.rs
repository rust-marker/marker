#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(clippy::exhaustive_enums)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unused_self)] // `self` is needed to change the behavior later
#![allow(clippy::missing_panics_doc)] // Temporary allow for `todo!`s
#![allow(clippy::new_without_default)] // Not very helpful as `new` is almost always cfged
#![cfg_attr(not(feature = "driver-api"), allow(dead_code))]

pub static MARKER_API_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod ast;
pub mod context;
pub mod diagnostic;
pub mod interface;
pub mod lint;

#[doc(hidden)]
pub mod ffi;

pub use context::AstContext;
pub use interface::{LintPassInfo, LintPassInfoBuilder};

/// A [`LintPass`] visits every node like a `Visitor`. The difference is that a
/// [`LintPass`] provides some additional information about the implemented lints.
/// The adapter will walk through the entire AST once and give each node to the
/// registered [`LintPass`]es.
pub trait LintPass {
    fn info(&self) -> LintPassInfo;

    fn check_item<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _item: ast::item::ItemKind<'ast>) {}
    fn check_field<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _field: &'ast ast::item::Field<'ast>) {}
    fn check_variant<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _variant: &'ast ast::item::EnumVariant<'ast>) {}
    fn check_body<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _body: &'ast ast::item::Body<'ast>) {}
    fn check_stmt<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _stmt: ast::stmt::StmtKind<'ast>) {}
    fn check_expr<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _expr: ast::expr::ExprKind<'ast>) {}
}
