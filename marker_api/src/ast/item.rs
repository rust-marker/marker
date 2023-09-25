use std::{fmt::Debug, marker::PhantomData};

use crate::{
    diagnostic::EmissionNode,
    private::Sealed,
    span::{HasSpan, Ident, Span},
    CtorBlocker,
};

use super::expr::ExprKind;
use super::{HasNodeId, ItemId, SpanId};

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
crate::ast::impl_identifiable_for!(ItemKind<'ast>);
impl<'ast> crate::private::Sealed for ItemKind<'ast> {}

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
crate::ast::impl_identifiable_for!(AssocItemKind<'ast>);
impl<'ast> crate::private::Sealed for AssocItemKind<'ast> {}

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
crate::ast::impl_identifiable_for!(ExternItemKind<'ast>);
impl<'ast> crate::private::Sealed for ExternItemKind<'ast> {}

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

        $crate::ast::impl_identifiable_for!($self_name<'ast>, use $crate::ast::item::ItemData);
        impl $crate::private::Sealed for $self_name<'_> {}

        impl<'ast> From<&'ast $self_name<'ast>> for crate::ast::item::ItemKind<'ast> {
            fn from(value: &'ast $self_name<'ast>) -> Self {
                $crate::ast::item::ItemKind::$enum_name(value)
            }
        }
    };
}

use impl_item_data;

#[cfg(feature = "driver-api")]
impl<'ast> CommonItemData<'ast> {
    pub fn new(id: ItemId, span: SpanId, ident: Ident<'ast>) -> Self {
        Self {
            id,
            span,
            vis: Visibility::new(id),
            ident,
        }
    }
}

/// This struct represents the visibility of an item.
///
/// Currently, it's only a placeholder until a proper representation is implemented.
/// rust-marker/marker#26 tracks the task of implementing this. You're welcome to
/// leave any comments in that issue.
#[repr(C)]
pub struct Visibility<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    _item_id: ItemId,
}

impl<'ast> Debug for Visibility<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Visibility {{ /* WIP: See rust-marker/marker#26 */}}")
            .finish()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Visibility<'ast> {
    pub fn new(item_id: ItemId) -> Self {
        Self {
            _lifetime: PhantomData,
            _item_id: item_id,
        }
    }
}

/// A body represents the expression of items.
///
/// Bodies act like a barrier between the item and expression level. When items
/// are requested, only the item information is retrieved and converted. Any
/// expression parts of these items are wrapped into a body, identified via a
/// [`BodyId`](`super::BodyId`). The body and its content will only be converted
/// request.
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
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_item_struct_size() {
        // These sizes are allowed to change, this is just a check to have a
        // general overview and to prevent accidental changes
        assert_eq!(56, size_of::<ModItem<'_>>(), "ModItem");
        assert_eq!(48, size_of::<ExternCrateItem<'_>>(), "ExternCrateItem");
        assert_eq!(64, size_of::<UseItem<'_>>(), "UseItem");
        assert_eq!(80, size_of::<StaticItem<'_>>(), "StaticItem");
        assert_eq!(72, size_of::<ConstItem<'_>>(), "ConstItem");
        assert_eq!(144, size_of::<FnItem<'_>>(), "FnItem");
        assert_eq!(112, size_of::<TyAliasItem<'_>>(), "TyAliasItem");
        assert_eq!(96, size_of::<StructItem<'_>>(), "StructItem");
        assert_eq!(88, size_of::<EnumItem<'_>>(), "EnumItem");
        assert_eq!(88, size_of::<UnionItem<'_>>(), "UnionItem");
        assert_eq!(112, size_of::<TraitItem<'_>>(), "TraitItem");
        assert_eq!(144, size_of::<ImplItem<'_>>(), "ImplItem");
        assert_eq!(64, size_of::<ExternBlockItem<'_>>(), "ExternBlockItem");
        assert_eq!(48, size_of::<UnstableItem<'_>>(), "UnstableItem");
    }
}
