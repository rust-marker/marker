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
    /// Returns `true` if the item is declared as public, without any restrictions.
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
    /// ```
    ///
    /// See [`Visibility::is_pub_in_path`] to detect pub declarations with a
    /// defined path.
    pub fn is_pub(&self) -> bool {
        matches!(self.kind, VisibilityKind::Public | VisibilityKind::DefaultPub)
    }

    /// Returns `true` if the item is declared as `pub(in ..)` with a path in brackets
    /// that defines the scope, where the item is visible.
    pub fn is_pub_in_path(&self) -> bool {
        matches!(self.kind, VisibilityKind::Path(_))
    }

    /// Returns `true` if the visibility is declared as `pub(crate)`. This is a
    /// special case of the `pub(<path>)` visibility.
    ///
    /// This function checks if the visibility is restricted and the defined path
    /// belongs to the root module of the crate. Meaning, that this can also be `true`,
    /// if the visibility uses `pub(super)` to travel up to the crate root.
    // Ignore, since the `in crate::example_1` path doesn't work for doc tests
    /// ```ignore
    /// // lib.rs
    ///
    /// mod example_1 {
    ///     // Returns `false` since no visibility is declared
    ///     fn foo() {}
    ///
    ///     // Returns `false` since the item is not visible from the root of the crate.
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
    /// // Returns `false` since the visibility is not restricted
    /// fn example_in_root() {}
    /// ```
    pub fn is_pub_crate(&self) -> bool {
        matches!(self.kind, VisibilityKind::Crate(_))
    }

    /// Returns `true` if a visibility is the default visibility, meaning that it wasn't
    /// declared.
    pub fn is_default(&self) -> bool {
        matches!(self.kind, VisibilityKind::Default(_) | VisibilityKind::DefaultPub)
    }

    /// Returns the [`ItemId`] of the module where this item is visible in, if the
    /// visibility is defined to be public inside a specified path.
    ///
    /// See [`Visibility::module_id`] to get the `ItemId`, even if the item or
    /// field uses the default visibility.
    pub fn pub_with_path_module_id(&self) -> Option<ItemId> {
        match self.kind {
            VisibilityKind::Path(id) | VisibilityKind::Crate(id) => Some(id),
            _ => None,
        }
    }

    /// Returns the [`ItemId`] of the module that this item or field is visible in.
    /// It will return `None`, if the item is public, as the visibility extends even past
    /// the root module of the crate.
    ///
    /// This function also works for items which have the default visibility, of the
    /// module they are declared in.
    ///
    /// ```
    /// mod scope {
    ///     // Returns `None` since this is even visible outside the current crate
    ///     pub fn turtle() {}
    ///     
    ///     // Returns the `ItemId` of the root module of the crate
    ///     pub(crate) fn shark() {}
    ///
    ///     // Returns the `ItemId` of the module it is declared in
    ///     fn dolphin() {}
    /// }
    /// ```
    ///
    /// Note that this only returns the [`ItemId`] that this item is visible in
    /// based on the declared visibility. The item might be reexported, which can
    /// increase the visibility.
    pub fn module_id(&self) -> Option<ItemId> {
        match self.kind {
            VisibilityKind::Path(id) | VisibilityKind::Crate(id) | VisibilityKind::Default(id) => Some(id),
            _ => None,
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
    /// The visibility is restricted to a specific module using `pub(<path>)`.
    /// The module, targeted by the path is identified by the [`ItemId`].
    /// The `pub(crate)` has it's own variant in this struct.
    Path(ItemId),
    /// The visibility is restricted to the root module of the crate. The [`ItemId`]
    /// identifies the root module.
    Crate(ItemId),
    /// The items doesn't have a declared visibility. The default is restricted to
    /// a module, identified by the stored [`ItemId`]
    Default(ItemId),
    /// For items which are `pub` by default, like trait functions or enum variants
    DefaultPub,
}
