use std::fmt::Debug;

use crate::{
    common::{HasNodeId, ItemId, SpanId},
    context::with_cx,
    diagnostic::EmissionNode,
    ffi::FfiOption,
    private::Sealed,
    span::{HasSpan, Ident, Span},
    CtorBlocker,
};

use super::expr::ExprKind;

// Item implementations
mod extern_crate_item;
pub use self::extern_crate_item::ExternCrateItem;
mod mod_item;
pub use mod_item::ModItem;
mod static_item;
pub use self::static_item::StaticItem;
mod use_item;
pub use self::use_item::*;
mod const_item;
pub use self::const_item::ConstItem;
mod fn_item;
pub use fn_item::*;
mod ty_alias_item;
pub use ty_alias_item::*;
mod adt_item;
pub use adt_item::*;
mod trait_item;
pub use trait_item::*;
mod impl_item;
pub use impl_item::*;
mod extern_block_item;
pub use extern_block_item::*;
mod unstable_item;
pub use unstable_item::*;

/// This trait combines methods, which are common between all items.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait ItemData<'ast>: Debug + EmissionNode<'ast> + HasSpan<'ast> + HasNodeId + Sealed {
    /// Returns the [`ItemId`] of this item. This is a unique identifier used for comparison
    /// and to request items from the [`MarkerContext`](`crate::context::MarkerContext`).
    fn id(&self) -> ItemId;

    /// The [`Visibility`] of this item.
    fn visibility(&self) -> &Visibility<'ast>;

    /// This function can return [`None`] if the item was generated and has no real name
    fn ident(&self) -> Option<&Ident<'ast>>;

    /// Returns this item wrapped in it's [`ExprKind`] variant.
    ///
    /// In function parameters, it's recommended to use `Into<ItemKind<'ast>>`
    /// as a bound to support all items and `ItemKind<'ast>` as parameters.
    fn as_item(&'ast self) -> ItemKind<'ast>;

    /// The attributes attached to this item.
    ///
    /// Currently, it's only a placeholder until a proper representation is implemented.
    /// rust-marker/marker#51 tracks the task of implementing this. You're welcome to
    /// leave any comments in that issue.
    fn attrs(&self); // FIXME: Add return type: -> &'ast [&'ast dyn Attribute<'ast>];
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ItemKind<'ast> {
    Mod(&'ast ModItem<'ast>),
    ExternCrate(&'ast ExternCrateItem<'ast>),
    Use(&'ast UseItem<'ast>),
    Static(&'ast StaticItem<'ast>),
    Const(&'ast ConstItem<'ast>),
    Fn(&'ast FnItem<'ast>),
    TyAlias(&'ast TyAliasItem<'ast>),
    Struct(&'ast StructItem<'ast>),
    Enum(&'ast EnumItem<'ast>),
    Union(&'ast UnionItem<'ast>),
    Trait(&'ast TraitItem<'ast>),
    Impl(&'ast ImplItem<'ast>),
    ExternBlock(&'ast ExternBlockItem<'ast>),
    Unstable(&'ast UnstableItem<'ast>),
}

impl<'ast> ItemKind<'ast> {
    impl_item_type_fn!(ItemKind: id() -> ItemId);
    impl_item_type_fn!(ItemKind: span() -> &Span<'ast>);
    impl_item_type_fn!(ItemKind: visibility() -> &Visibility<'ast>);
    impl_item_type_fn!(ItemKind: ident() -> Option<&Ident<'ast>>);
    impl_item_type_fn!(ItemKind: attrs() -> ());
}

crate::span::impl_spanned_for!(ItemKind<'ast>);
crate::common::impl_identifiable_for!(ItemKind<'ast>);

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum AssocItemKind<'ast> {
    TyAlias(&'ast TyAliasItem<'ast>, CtorBlocker),
    Const(&'ast ConstItem<'ast>, CtorBlocker),
    Fn(&'ast FnItem<'ast>, CtorBlocker),
}

impl<'ast> AssocItemKind<'ast> {
    impl_item_type_fn!(AssocItemKind: id() -> ItemId);
    impl_item_type_fn!(AssocItemKind: span() -> &Span<'ast>);
    impl_item_type_fn!(AssocItemKind: visibility() -> &Visibility<'ast>);
    impl_item_type_fn!(AssocItemKind: ident() -> Option<&Ident<'ast>>);
    impl_item_type_fn!(AssocItemKind: attrs() -> ());
    impl_item_type_fn!(AssocItemKind: as_item() -> ItemKind<'ast>);
    // FIXME: Potentially add a field to the items to optionally store the owner id
}

crate::span::impl_spanned_for!(AssocItemKind<'ast>);
crate::common::impl_identifiable_for!(AssocItemKind<'ast>);

impl<'ast> From<AssocItemKind<'ast>> for ItemKind<'ast> {
    fn from(value: AssocItemKind<'ast>) -> Self {
        match value {
            AssocItemKind::TyAlias(item, ..) => ItemKind::TyAlias(item),
            AssocItemKind::Const(item, ..) => ItemKind::Const(item),
            AssocItemKind::Fn(item, ..) => ItemKind::Fn(item),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExternItemKind<'ast> {
    Static(&'ast StaticItem<'ast>, CtorBlocker),
    Fn(&'ast FnItem<'ast>, CtorBlocker),
}

impl<'ast> ExternItemKind<'ast> {
    impl_item_type_fn!(ExternItemKind: id() -> ItemId);
    impl_item_type_fn!(ExternItemKind: span() -> &Span<'ast>);
    impl_item_type_fn!(ExternItemKind: visibility() -> &Visibility<'ast>);
    impl_item_type_fn!(ExternItemKind: ident() -> Option<&Ident<'ast>>);
    impl_item_type_fn!(ExternItemKind: attrs() -> ());
    impl_item_type_fn!(ExternItemKind: as_item() -> ItemKind<'ast>);
}

crate::span::impl_spanned_for!(ExternItemKind<'ast>);
crate::common::impl_identifiable_for!(ExternItemKind<'ast>);

impl<'ast> From<ExternItemKind<'ast>> for ItemKind<'ast> {
    fn from(value: ExternItemKind<'ast>) -> Self {
        match value {
            ExternItemKind::Static(item, ..) => ItemKind::Static(item),
            ExternItemKind::Fn(item, ..) => ItemKind::Fn(item),
        }
    }
}

/// Until [trait upcasting](https://github.com/rust-lang/rust/issues/65991) has been implemented
/// and stabilized we need this to call [`ItemData`] functions for [`ItemKind`].
macro_rules! impl_item_type_fn {
    (ItemKind: $method:ident () -> $return_ty:ty) => {
        impl_item_type_fn!((ItemKind) $method() -> $return_ty,
            Mod, ExternCrate, Use, Static, Const, Fn, TyAlias, Struct, Enum,
            Union, Trait, Impl, ExternBlock, Unstable
        );
    };
    (AssocItemKind: $method:ident () -> $return_ty:ty) => {
        impl_item_type_fn!((AssocItemKind) $method() -> $return_ty,
            TyAlias, Const, Fn
        );
    };
    (ExternItemKind: $method:ident () -> $return_ty:ty) => {
        impl_item_type_fn!((ExternItemKind) $method() -> $return_ty,
            Static, Fn
        );
    };
    (($self:ident) $method:ident () -> $return_ty:ty $(, $item:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $($self::$item(data, ..) => data.$method(),)*
            }
        }
    };
}

use impl_item_type_fn;

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
struct CommonItemData<'ast> {
    id: ItemId,
    span: SpanId,
    vis: Visibility<'ast>,
    ident: Ident<'ast>,
}

macro_rules! impl_item_data {
    ($self_name:ident, $enum_name:ident) => {
        impl<'ast> super::ItemData<'ast> for $self_name<'ast> {
            fn id(&self) -> crate::ast::item::ItemId {
                self.data.id
            }

            fn visibility(&self) -> &crate::ast::item::Visibility<'ast> {
                &self.data.vis
            }

            fn ident(&self) -> Option<&crate::span::Ident<'ast>> {
                Some(&self.data.ident)
            }

            fn as_item(&'ast self) -> crate::ast::item::ItemKind<'ast> {
                $crate::ast::item::ItemKind::$enum_name(self)
            }

            fn attrs(&self) {}
        }

        impl<'ast> $crate::span::HasSpan<'ast> for $self_name<'ast> {
            fn span(&self) -> &crate::span::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.span(self.data.span))
            }
        }

        $crate::common::impl_identifiable_for!($self_name<'ast>, use $crate::ast::item::ItemData);
        impl $crate::private::Sealed for $self_name<'_> {}

        impl<'ast> From<&'ast $self_name<'ast>> for crate::ast::item::ItemKind<'ast> {
            fn from(value: &'ast $self_name<'ast>) -> Self {
                $crate::ast::item::ItemKind::$enum_name(value)
            }
        }
    };
}

use impl_item_data;

/// The declared visibility of an item or field.
///
/// Note that this is only the syntactic visibility. The item or field might be
/// reexported with a higher visibility, or have a high default visibility.
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
    #[cfg_attr(feature = "driver-api", builder(setter(into), default))]
    span: FfiOption<SpanId>,
    sem: crate::sem::Visibility<'ast>,
}

impl<'ast> Visibility<'ast> {
    /// The [`Span`] of the visibility, if it has been declared.
    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.copy().map(|span| with_cx(self, |cx| cx.span(span)))
    }

    /// This function returns the semantic representation for the [`Visibility`]
    /// of this item. That visibility can be used to check if the item is public
    /// or restricted to specific modules.
    pub fn semantics(&self) -> &crate::sem::Visibility<'ast> {
        &self.sem
    }
}

/// A body represents the expression of items.
///
/// Bodies act like a barrier between the item and expression level. When items
/// are requested, only the item information is retrieved and converted. Any
/// expression parts of these items are wrapped into a body, identified via a
/// [`BodyId`](`crate::common::BodyId`). The body and its content will only be
/// converted request.
#[repr(C)]
#[derive(Debug)]
pub struct Body<'ast> {
    owner: ItemId,
    expr: ExprKind<'ast>,
}

impl<'ast> Body<'ast> {
    pub fn owner(&self) -> ItemId {
        self.owner
    }

    /// The expression wrapped by this body. In most cases this will be a
    /// [block expression](`crate::ast::expr::BlockExpr`).
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Body<'ast> {
    pub fn new(owner: ItemId, expr: ExprKind<'ast>) -> Self {
        Self { owner, expr }
    }
}

#[cfg(all(test, target_arch = "x86_64", target_pointer_width = "64"))]
mod test {
    use crate::test::assert_size_of;

    use super::*;
    use expect_test::expect;

    #[test]
    fn test_item_struct_size() {
        // These sizes are allowed to change, this is just a check to have a
        // general overview and to prevent accidental changes
        assert_size_of::<ModItem<'_>>(&expect!["80"]);
        assert_size_of::<ExternCrateItem<'_>>(&expect!["72"]);
        assert_size_of::<UseItem<'_>>(&expect!["88"]);
        assert_size_of::<StaticItem<'_>>(&expect!["104"]);
        assert_size_of::<ConstItem<'_>>(&expect!["96"]);
        assert_size_of::<FnItem<'_>>(&expect!["168"]);
        assert_size_of::<TyAliasItem<'_>>(&expect!["136"]);
        assert_size_of::<StructItem<'_>>(&expect!["120"]);
        assert_size_of::<EnumItem<'_>>(&expect!["112"]);
        assert_size_of::<UnionItem<'_>>(&expect!["112"]);
        assert_size_of::<TraitItem<'_>>(&expect!["136"]);
        assert_size_of::<ImplItem<'_>>(&expect!["168"]);
        assert_size_of::<ExternBlockItem<'_>>(&expect!["88"]);
        assert_size_of::<UnstableItem<'_>>(&expect!["72"]);
    }
}
