#![doc = include_str!("../README.md")]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

use ast::item::{ExternCrateItem, ItemType, ModItem, StaticItem, UseDeclItem};
use context::AstContext;
use lint::Lint;

#[doc(hidden)]
pub static LINTER_API_VERSION: &str = env!("CARGO_PKG_VERSION");
#[doc(hidden)]
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

    fn check_item(&mut self, _cx: &'ast AstContext<'ast>, _item: ItemType<'ast>) {}

    fn check_mod(&mut self, _cx: &'ast AstContext<'ast>, _mod_item: &'ast ModItem<'ast>) {}

    fn check_extern_crate(&mut self, _cx: &'ast AstContext<'ast>, _extern_crate_item: &'ast ExternCrateItem<'ast>) {}

    fn check_use_decl(&mut self, _cx: &'ast AstContext<'ast>, _use_item: &'ast UseDeclItem<'ast>) {}

    fn check_static_item(&mut self, _cx: &'ast AstContext<'ast>, _item: &'ast StaticItem<'ast>) {}
}
