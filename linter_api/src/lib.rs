#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::trivially_copy_pass_by_ref)]

#[doc(hidden)]
pub static LINTER_API_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod ast;
pub mod context;
pub mod interface;
pub mod lint;

/// **!Unstable!**
///
/// This macro returns a list of all functions declared for the [`LintPass`] trait.
/// All references use the `'ast` lifetime, this needs to be provided in the given
/// scope. The first token is a dollar sign `$` in brackets to support macro creation
/// based on these declarations. The mutability of `self` is wrapped in brackets to
/// support optional matching.
///
/// The functions can be categorized as follows:
///
/// 1. Informative functions used to retrieve information from the [`LintPass`]
///    implementation. These functions take an unmutable reference to self and
///    require a manual implementation
/// 2. Check functions, which can be implemented to check specific nodes from the
///    AST. All of these are optional and have no return type. For us, they are
///    *fire and forget*.
///
/// [`for_each_lint_pass_fn`] can be used to process each item individually.
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

/// **!Unstable!**
///
/// This generates the [`for_each_lint_pass_fn`] macro.
#[macro_export]
#[doc(hidden)]
macro_rules! gen_for_each_lint_pass_fn {
    (($dollar:tt) $(fn $fn_name:ident(& $(($mut_:tt))? self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty;)+) => {
        /// **!Unstable!**
        ///
        /// This calls a macro for each function available in the [`LintPass`] trait. The following
        /// patterns can be used to match the two different types of functions currently defined for
        /// the trait. See [`lint_pass_fns`] for more information.
        /// ```
        /// macro_rules! lint_pass_macro {
        ///     (fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        ///         // TODO
        ///     };
        ///     (fn $fn_name:ident(&(mut) self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        ///         // TODO
        ///     };
        /// }
        /// ```
        #[macro_export]
        #[doc(hidden)]
        macro_rules! for_each_lint_pass_fn {
            ($dollar macro_name:path) => {
                $(
                    $dollar macro_name !(fn $fn_name(& $(($mut_))? self $(, $arg_name: $arg_ty)*) -> $ret_ty);
                )*
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
