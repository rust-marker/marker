use std::fmt::Debug;

use super::{Attribute, Ident, SimplePath, Span, Spanned, Symbol};

pub trait Item<'ast>: Debug {
    fn get_id(&self) -> ItemId;

    fn get_span(&'ast self) -> &'ast dyn Span<'ast>;

    fn get_vis(&self) -> &'ast Visibility<'ast>;

    /// This function can return `None` if the item was generated and has no real name
    fn get_ident(&'ast self) -> Option<&'ast Ident<'ast>>;

    fn get_kind(&'ast self) -> ItemKind<'ast>;

    fn get_attrs(&'ast self) -> &'ast [&dyn Attribute<'ast>];
}

/// Every item has an ID that can be used to retive that item or compair it to
/// another id. The ID's can change in between linting sessions.
///
/// The interal representation is currently based on rustc's `DefId`. This might
/// change in the future. The struct will continue to provide the current trait
/// implementations.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ItemId {
    index: usize,
    krate: usize,
}

#[cfg(feature = "driver-api")]
impl ItemId {
    #[must_use]
    pub fn new(index: usize, krate: usize) -> Self {
        Self { index, krate }
    }
}

/// The visibility of items.
///
/// See: <https://doc.rust-lang.org/reference/visibility-and-privacy.html>
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum VisibilityKind {
    /// Visible in the current module, equivialent to `pub(in self)` or no visibility
    PubSelf,
    PubCrate,
    /// FIXME: Add a path value to this
    PubPath,
    PubSuper,
}

pub type Visibility<'ast> = Spanned<'ast, VisibilityKind>;

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum ItemKind<'ast> {
    Mod(&'ast dyn Module<'ast>),
    /// An `extern crate` item, with an optional *original* create name. The given
    /// and used name is the identifier of the [`Item`].
    ExternCrate(Option<Symbol>),
    UseDeclaration(&'ast dyn SimplePath<'ast>, UseKind),
    Function,
    TypeAlias,
    Struct,
    Enumeration,
    Union,
    ConstantItem,
    StaticItem,
    Trait,
    Implementation,
    ExternBlock,
}

pub trait Module<'ast>: Debug {
    fn get_inner_attrs(&'ast self) -> [&dyn Attribute<'ast>];

    fn get_items(&'ast self) -> [&dyn Item<'ast>];
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UseKind {
    /// Single usages like `use foo::bar` a list of multiple usages like
    /// `use foo::{bar, baz}` will be desugured to `use foo::bar; use foo::baz;`
    Single,
    /// A glob import like `use foo::*`
    Glob,
}
