use std::{fmt::Debug, marker::PhantomData};

use super::expr::ExprKind;
use super::{ItemId, Span, SymbolId};

// Item implementations
mod extern_crate_item;
pub use self::extern_crate_item::ExternCrateItem;
mod mod_item;
pub use mod_item::ModItem;
mod static_item;
pub use self::static_item::StaticItem;
mod use_decl_item;
pub use self::use_decl_item::*;
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

pub trait ItemData<'ast>: Debug {
    /// Returns the [`ItemId`] of this item. This is a unique identifier used for comparison
    /// and to request items from the [`AstContext`][`crate::context::AstContext`].
    fn id(&self) -> ItemId;

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    fn span(&self) -> &Span<'ast>;

    /// The visibility of this item.
    fn visibility(&self) -> &Visibility<'ast>;

    /// This function can return `None` if the item was generated and has no real name
    fn name(&self) -> Option<String>;

    /// This returns this [`ItemData`] instance as a [`ItemKind`]. This can be useful for
    /// functions that take [`ItemKind`] as a parameter. For general function calls it's better
    /// to call them directoly on the item, instead of converting it to a [`ItemKind`] first.
    fn as_item(&'ast self) -> ItemKind<'ast>;

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
    impl_item_type_fn!(ItemKind: name() -> Option<String>);
    impl_item_type_fn!(ItemKind: attrs() -> ());
}

#[non_exhaustive]
#[derive(Debug)]
pub enum AssocItemKind<'ast> {
    TyAlias(&'ast TyAliasItem<'ast>),
    Const(&'ast ConstItem<'ast>),
    Fn(&'ast FnItem<'ast>),
}

impl<'ast> AssocItemKind<'ast> {
    impl_item_type_fn!(AssocItemKind: id() -> ItemId);
    impl_item_type_fn!(AssocItemKind: span() -> &Span<'ast>);
    impl_item_type_fn!(AssocItemKind: visibility() -> &Visibility<'ast>);
    impl_item_type_fn!(AssocItemKind: name() -> Option<String>);
    impl_item_type_fn!(AssocItemKind: attrs() -> ());
    impl_item_type_fn!(AssocItemKind: as_item() -> ItemKind<'ast>);
    // FIXME: Potentualy add a field to the items to optionally store the owner id
}

impl<'ast> From<AssocItemKind<'ast>> for ItemKind<'ast> {
    fn from(value: AssocItemKind<'ast>) -> Self {
        match value {
            AssocItemKind::TyAlias(item) => ItemKind::TyAlias(item),
            AssocItemKind::Const(item) => ItemKind::Const(item),
            AssocItemKind::Fn(item) => ItemKind::Fn(item),
        }
    }
}

impl<'ast> TryFrom<&ItemKind<'ast>> for AssocItemKind<'ast> {
    type Error = ();

    fn try_from(value: &ItemKind<'ast>) -> Result<Self, Self::Error> {
        match value {
            ItemKind::TyAlias(item) => Ok(AssocItemKind::TyAlias(item)),
            ItemKind::Const(item) => Ok(AssocItemKind::Const(item)),
            ItemKind::Fn(item) => Ok(AssocItemKind::Fn(item)),
            _ => Err(()),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ExternItemKind<'ast> {
    Static(&'ast StaticItem<'ast>),
    Fn(&'ast FnItem<'ast>),
}

impl<'ast> ExternItemKind<'ast> {
    impl_item_type_fn!(ExternItemKind: id() -> ItemId);
    impl_item_type_fn!(ExternItemKind: span() -> &Span<'ast>);
    impl_item_type_fn!(ExternItemKind: visibility() -> &Visibility<'ast>);
    impl_item_type_fn!(ExternItemKind: name() -> Option<String>);
    impl_item_type_fn!(ExternItemKind: attrs() -> ());
    impl_item_type_fn!(ExternItemKind: as_item() -> ItemKind<'ast>);
}

impl<'ast> From<ExternItemKind<'ast>> for ItemKind<'ast> {
    fn from(value: ExternItemKind<'ast>) -> Self {
        match value {
            ExternItemKind::Static(item) => ItemKind::Static(item),
            ExternItemKind::Fn(item) => ItemKind::Fn(item),
        }
    }
}

impl<'ast> TryFrom<ItemKind<'ast>> for ExternItemKind<'ast> {
    type Error = ();

    fn try_from(value: ItemKind<'ast>) -> Result<Self, Self::Error> {
        match value {
            ItemKind::Static(item) => Ok(ExternItemKind::Static(item)),
            ItemKind::Fn(item) => Ok(ExternItemKind::Fn(item)),
            _ => Err(()),
        }
    }
}

/// Until [trait upcasting](https://github.com/rust-lang/rust/issues/65991) has been implemented
/// and stabalized we need this to call [`ItemData`] functions for [`ItemKind`].
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
                $($self::$item(data) => data.$method(),)*
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
    vis: Visibility<'ast>,
    name: SymbolId,
}

macro_rules! impl_item_data {
    ($self_name:ident, $enum_name:ident) => {
        impl<'ast> super::ItemData<'ast> for $self_name<'ast> {
            fn id(&self) -> crate::ast::item::ItemId {
                self.data.id
            }

            fn span(&self) -> &crate::ast::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.get_span(self.data.id))
            }

            fn visibility(&self) -> &crate::ast::item::Visibility<'ast> {
                &self.data.vis
            }

            fn name(&self) -> Option<String> {
                Some($crate::context::with_cx(self, |cx| cx.symbol_str(self.data.name)))
            }

            fn as_item(&'ast self) -> crate::ast::item::ItemKind<'ast> {
                $crate::ast::item::ItemKind::$enum_name(self)
            }

            fn attrs(&self) {
                todo!()
            }
        }
    };
}

use impl_item_data;

#[cfg(feature = "driver-api")]
impl<'ast> CommonItemData<'ast> {
    pub fn new(id: ItemId, name: SymbolId) -> Self {
        Self {
            id,
            vis: Visibility::new(id),
            name,
        }
    }
}

/// FIXME: Add function as  discussed in <https://github.com/rust-marker/design/issues/22>
/// this will require new driver callback functions
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Visibility<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    _item_id: ItemId,
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

    /// The expression wrapped by this body. In some cases, like for functions
    /// this expression is guaranteed to be a
    /// [block expression](`crate::ast::expr::BlockExpr`).
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}
