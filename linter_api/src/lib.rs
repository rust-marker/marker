#![warn(clippy::pedantic, clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

use ast::item::ItemType;
use context::Context;
use lint::Lint;

pub static LINTER_API_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub mod ast;
pub mod context;
pub mod interface;
pub mod lint;

/// A [`LintPass`] visits every node like a `Visitor`. The difference is that a
/// [`LintPass`] provides some additional information about the implemented lints.
/// The adapter will walk through the entire AST once and give each node to the
/// registered [`LintPass`]es.
pub trait LintPass<'ast> {
    fn registered_lints(&self) -> Vec<&'static Lint>;

    fn check_item(&mut self, _cx: &'ast Context<'ast>, _item: ItemType<'ast>) {}
}
