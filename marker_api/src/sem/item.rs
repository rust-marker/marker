use std::marker::PhantomData;

use crate::common::ItemId;

/// The declared visibility of an item or field.
///
/// ```
/// // An item without a visibility
/// fn moon() {}
///
/// // A public item
/// pub fn sun() {}
///
/// // An item with a restricted scope
/// pub(crate) fn star() {}
///
/// pub trait Planet {
///     // An item without a visibility. But it still has the semantic visibility
///     // of `pub` as this is inside a trait declaration.
///     fn mass();
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct Visibility<'ast> {
    #[cfg_attr(feature = "driver-api", builder(setter(skip), default))]
    _lifetime: PhantomData<&'ast ()>,
    kind: VisibilityKind,
}

impl<'ast> Visibility<'ast> {
    /// Returns `true` if the item is public, meaning that it can be visible outside
    /// the crate.
    ///
    /// ```
    /// // This returns `true`
    /// pub fn unicorn() {}
    ///
    /// // This returns `false`, since the visibility is restricted to a specified path.
    /// pub(crate) fn giraffe() {}
    ///
    /// // This returns `false`, since the visibility is not defined
    /// fn dragon() {}
    ///
    /// pub trait MythicalCreature {
    ///     // This returns `true`, since the default visibility for trait items is public
    ///     fn name() -> &'static str;
    /// }
    /// ```
    pub fn is_pub(&self) -> bool {
        self.scope().is_none()
    }

    /// Returns `true` if the item is visible in the entire crate. This is
    /// the case for items declared as `pub(crate)`. Items declared in the root
    /// module of the crate or specify the path of the root module are also
    /// scoped to the entire crate.
    ///
    /// ```
    /// // lib.rs
    ///
    /// mod example_1 {
    ///     // Returns `false` since it's only visible in `crate::example_1`
    ///     fn foo() {}
    ///
    ///     // Returns `false` since it's only visible in `crate::example_1`
    ///     pub(in crate::example_1) fn bar() {}
    ///
    ///     // Returns `true` as the visibility is restricted to the root of the crate.
    ///     pub(crate) fn baz() {}
    ///
    ///     // Returns `true` as the visibility is restricted to `super` which is the
    ///     // root of the crate.
    ///     pub(super) fn boo() {}
    /// }
    ///
    /// // Returns `true` since this item is at the root of the crate and the default
    /// // visibility is therefore `pub(crate)`
    /// fn example_in_root() {}
    ///
    /// # fn main() {}
    /// ```
    pub fn is_crate_scoped(&self) -> bool {
        matches!(self.kind, VisibilityKind::Crate(_) | VisibilityKind::DefaultCrate(_))
    }

    /// Returns `true` if a visibility is the default visibility, meaning that it wasn't
    /// declared.
    pub fn is_default(&self) -> bool {
        match self.kind {
            VisibilityKind::DefaultPub | VisibilityKind::DefaultCrate(_) | VisibilityKind::Default(_) => true,
            VisibilityKind::Public | VisibilityKind::Crate(_) | VisibilityKind::Path(_) => false,
        }
    }

    /// Returns the [`ItemId`] of the module that this item or field is visible in.
    /// It will return `None`, if the item is public, as the visibility extends past
    /// the root module of the crate.
    ///
    /// This function also works for items which have the default visibility, of the
    /// module they are declared in.
    ///
    /// ```
    /// // lib.rs
    ///
    /// mod scope {
    ///     // Returns `None` since this is even visible outside the current crate
    ///     pub fn turtle() {}
    ///     
    ///     // Returns the `ItemId` of the root module of the crate
    ///     pub(crate) fn shark() {}
    ///
    ///     // Returns the `ItemId` of the module it is declared in, in this case `crate::scope`
    ///     fn dolphin() {}
    /// }
    /// ```
    pub fn scope(&self) -> Option<ItemId> {
        match self.kind {
            VisibilityKind::Path(id)
            | VisibilityKind::Crate(id)
            | VisibilityKind::DefaultCrate(id)
            | VisibilityKind::Default(id) => Some(id),
            VisibilityKind::Public | VisibilityKind::DefaultPub => None,
        }
    }

    // FIXME(xFrednet): Implement functions to check if an item is visible from a
    // given `ItemId`. This can be done once rust-marker/marker#242 is implemented.
}

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum VisibilityKind {
    /// The item is declared as `pub` without any restrictions
    Public,
    /// For items which are `pub` by default, like trait functions or enum variants
    DefaultPub,
    /// The visibility is restricted to the root module of the crate. The [`ItemId`]
    /// identifies the root module.
    Crate(ItemId),
    /// The items doesn't have a declared visibility. The default is visible in the
    /// entire crate. The [`ItemId`] identifies the root module.
    DefaultCrate(ItemId),
    /// The visibility is restricted to a specific module using `pub(<path>)`.
    /// The module, targeted by the path is identified by the [`ItemId`].
    /// The `pub(crate)` has it's own variant in this struct.
    Path(ItemId),
    /// The items doesn't have a declared visibility. The default is restricted to
    /// a module, identified by the stored [`ItemId`]
    Default(ItemId),
}
