#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::trivially_copy_pass_by_ref)]

#[doc(hidden)]
pub static LINTER_API_VERSION: &str = env!("CARGO_PKG_VERSION");
#[doc(hidden)]
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub mod ast;
pub mod context;
pub mod interface;
pub mod lint;

/// This macro returns a list of all functions declared for lintpasses. The mutability
/// of self is is brackets, to support optional mutability matching
#[macro_export]
#[doc(hidden)]
macro_rules! lint_pass_fns {
    ($name:path) => {
        $name !(($)
            fn registered_lints(&self) -> Box<[&'static $crate::lint::Lint]>;

            fn check_item(
                &(mut) self,
                _cx: &'ast $crate::context::AstContext<'ast>,
                _item: $crate::ast::item::ItemType<'ast>) -> ();
            fn check_mod(
                &(mut) self,
                _cx: &'ast $crate::context::AstContext<'ast>,
                _mod_item: &'ast $crate::ast::item::ModItem<'ast>) -> ();
            fn check_extern_crate(
                &(mut) self,
                _cx: &'ast $crate::context::AstContext<'ast>,
                _extern_crate_item: &'ast $crate::ast::item::ExternCrateItem<'ast>) -> ();
            fn check_use_decl(
                &(mut) self,
                _cx: &'ast $crate::context::AstContext<'ast>,
                _use_item: &'ast $crate::ast::item::UseDeclItem<'ast>) -> ();
            fn check_static_item(
                &(mut) self,
                _cx: &'ast $crate::context::AstContext<'ast>,
                _item: &'ast $crate::ast::item::StaticItem<'ast>) -> ();
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! gen_for_each_lint_pass_fn {
    (($dollar:tt) $(fn $fn_name:ident(& $(($mutability:tt))? self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty;)+) => {
        /// This calls a macro for each function available in the [`LintPass`] trait.
        /// The given macro can use the following template:
        /// ```
        /// macro_rules! lint_pass_fn {
        ///     (fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        ///         // TODO
        ///     };
        ///     (fn $fn_name:ident(&(mut) self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        ///         // TODO
        ///     };
        /// }
        /// ```
        ///
        /// Note that this macro is not part of the stable ABI, it might be changed or expanded
        /// in the future.
        #[macro_export]
        #[doc(hidden)]
        macro_rules! for_each_lint_pass_fn {
            ($dollar macro_name:path) => {
                $(
                    $dollar macro_name !(fn $fn_name(& $(($mutability))? self $(, $arg_name: $arg_ty)*) -> $ret_ty);
                )*
                // Pass $ as agument, this is fun ...
            }
        }
    };
}
lint_pass_fns!(crate::gen_for_each_lint_pass_fn);

/// A [`LintPass`] visits every node like a `Visitor`. The difference is that a
/// [`LintPass`] provides some additional information about the implemented lints.
/// The adapter will walk through the entire AST once and give each node to the
/// registered [`LintPass`]es.
pub trait LintPass<'ast> {
    for_each_lint_pass_fn!(crate::decl_lint_pass_fn);
}

/// This macro currently expects that all declarations taken `&self` have to be
/// implemented while all taking `&mut self` have an empty default implementation.
#[doc(hidden)]
macro_rules! decl_lint_pass_fn {
    (fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        fn $fn_name(&self $(,$arg_name: $arg_ty)*) -> $ret_ty;
    };
    (fn $fn_name:ident(&(mut) self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        fn $fn_name(&mut self $(,$arg_name: $arg_ty)*) -> $ret_ty {}
    };
}
use decl_lint_pass_fn;
