#![feature(rustc_private)]
#![warn(clippy::pedantic, clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

use ast::item::Item;
use lint::Lint;

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_attr;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_index;
extern crate rustc_infer;
extern crate rustc_lexer;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_parse;
extern crate rustc_parse_format;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;
extern crate rustc_typeck;

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

    fn check_item(&mut self, _: &'ast dyn Item<'ast>) {}
}
