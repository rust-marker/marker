#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unused_self)] // `self` is needed to change the behavior later
#![allow(clippy::missing_panics_doc)] // Temporary allow for `todo!`s
#![allow(clippy::new_without_default)] // Not very helpful as `new` is almost always cfged
#![cfg_attr(not(feature = "driver-api"), allow(dead_code))]
// FIXME(#26): this is commented out because it is the lint that we want to enable
// here makes sense only on a public items, but it triggers of private/pub(crate)
// methods today. There isn't a way to inspect the item visibility in this lint's
// impl yet. Once #26 is done and lint impl ignores private functions we may enable
// this lint.
// #![cfg_attr(
//     marker,
//     feature(register_tool),
//     register_tool(marker),
//     warn(marker::not_using_has_span_trait)
// )]

pub static MARKER_API_VERSION: &str = env!("CARGO_PKG_VERSION");

mod interface;
mod private;
pub use interface::*;
mod lint;
pub use lint::*;

pub mod ast;
pub mod common;
pub mod context;
pub mod diagnostic;
pub mod prelude;
pub mod sem;
pub mod span;

#[doc(hidden)]
pub mod ffi;

pub use context::MarkerContext;
pub use interface::{LintPassInfo, LintPassInfoBuilder};

/// A [`LintPass`] visits every node like a `Visitor`. The difference is that a
/// [`LintPass`] provides some additional information about the implemented lints.
/// The adapter will walk through the entire AST once and give each node to the
/// registered [`LintPass`]es.
pub trait LintPass {
    fn info(&self) -> LintPassInfo;

    fn check_item<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _item: ast::ItemKind<'ast>) {}
    fn check_field<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _field: &'ast ast::ItemField<'ast>) {}
    fn check_variant<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _variant: &'ast ast::EnumVariant<'ast>) {}
    fn check_body<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _body: &'ast ast::Body<'ast>) {}
    fn check_stmt<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _stmt: ast::StmtKind<'ast>) {}
    fn check_expr<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _expr: ast::ExprKind<'ast>) {}
}

/// This struct blocks the construction of enum variants, similar to the `#[non_exhaustive]`
/// attribute.
///
/// Marker uses enums extensively, like [`ItemKind`][ast::ItemKind] and
/// [`ExprKind`](ast::ExprKind). There can be `*Kind` enums that wrap other
/// `*Kind` enums. In those cases, this struct is used, to block the user from
/// constructing the variant manually. This allows tools to handle the variants
/// confidently without additional verification. An example for this would be the
/// [`LitExprKind::UnaryOp`](ast::LitExprKind::UnaryOp) variant.
///
/// This basically acts like a `#[non_exhaustive]` attribute, with the difference
/// that it also works on tuple variants. Attaching `#[non_exhaustive]` to a tuple
/// variant would make it private, which we don't want.
///
/// As a normal user, you can just ignore this instance as it holds no relevant
/// information for linting.
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct CtorBlocker {
    /// `#[repr(C)]` requires a field, to make this a proper type. This is just
    /// the smallest one.
    _data: u8,
}

impl std::fmt::Debug for CtorBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("..").finish()
    }
}

impl CtorBlocker {
    #[cfg_attr(feature = "driver-api", visibility::make(pub))]
    pub(crate) fn new() -> Self {
        Self { _data: 255 }
    }
}
