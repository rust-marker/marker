use std::fmt::Debug;

use crate::context::AstContext;

// Item implementations
mod extern_crate_item;
pub use self::extern_crate_item::ExternCrateItem;
mod mod_item;
pub use self::mod_item::ModItem;
mod static_item;
pub use self::static_item::StaticItem;
mod use_decl_item;
pub use self::use_decl_item::UseDeclItem;
mod const_item;
pub use self::const_item::ConstItem;

use super::{
    ty_old::{TyIdOld, TyOld},
    Abi, Asyncness, Attribute, BodyId, Constness, ItemId, Pattern, Safety, Span, Symbol, SymbolId,
};

pub trait ItemData<'ast>: Debug {
    /// Returns the [`ItemId`] of this item. This is a unique identifier used for comparison
    /// and to request items from the [`AstContext`][`crate::context::AstContext`].
    fn id(&self) -> ItemId;

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    fn span(&self) -> &'ast Span<'ast>;

    /// The visibility of this item.
    fn visibility(&self) -> &Visibility<'ast>;

    /// This function can return `None` if the item was generated and has no real name
    fn name(&self) -> Option<String>;

    /// This returns this [`ItemData`] instance as a [`ItemType`]. This can be useful for
    /// functions that take [`ItemType`] as a parameter. For general function calls it's better
    /// to call them directoly on the item, instead of converting it to a [`ItemType`] first.
    fn as_item(&'ast self) -> ItemType<'ast>;

    fn attrs(&self); // FIXME: Add return type: -> &'ast [&'ast dyn Attribute<'ast>];
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ItemType<'ast> {
    Mod(&'ast ModItem<'ast>),
    ExternCrate(&'ast ExternCrateItem<'ast>),
    UseDecl(&'ast UseDeclItem<'ast>),
    Static(&'ast StaticItem<'ast>),
    Const(&'ast ConstItem<'ast>),
    Function(&'ast dyn FunctionItem<'ast>),
    TypeAlias(&'ast dyn TypeAliasItem<'ast>),
    Struct(&'ast dyn StructItem<'ast>),
    Enum(&'ast dyn EnumItem<'ast>),
    Union(&'ast dyn UnionItem<'ast>),
    Trait(&'ast dyn TraitItem<'ast>),
    Impl(&'ast dyn ImplItem<'ast>),
    ExternBlock(&'ast dyn ExternBlockItem<'ast>),
}

impl<'ast> ItemType<'ast> {
    impl_item_type_fn!(id() -> ItemId);
    impl_item_type_fn!(span() -> &'ast Span<'ast>);
    impl_item_type_fn!(visibility() -> &Visibility<'ast>);
    impl_item_type_fn!(name() -> Option<String>);
    impl_item_type_fn!(attrs() -> ());
}

/// Until [trait upcasting](https://github.com/rust-lang/rust/issues/65991) has been implemented
/// and stabalized we need this to call [`ItemData`] functions for [`ItemType`].
macro_rules! impl_item_type_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_item_type_fn!($method() -> $return_ty,
            Mod, ExternCrate, UseDecl, Static, Const, Function,
            TypeAlias, Struct, Enum, Union, Trait, Impl, ExternBlock
        );
    };
    ($method:ident () -> $return_ty:ty $(, $item:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $(ItemType::$item(data) => data.$method(),)*
            }
        }
    };
}

use impl_item_type_fn;

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct CommonItemData<'ast> {
    cx: &'ast AstContext<'ast>,
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

            fn span(&self) -> &'ast crate::ast::Span<'ast> {
                self.data.cx.get_span(crate::ast::SpanOwner::Item(self.data.id))
            }

            fn visibility(&self) -> &crate::ast::item::Visibility<'ast> {
                &self.data.vis
            }

            fn name(&self) -> Option<String> {
                Some(self.data.cx.symbol_str(self.data.name))
            }

            fn as_item(&'ast self) -> crate::ast::item::ItemType<'ast> {
                $crate::ast::item::ItemType::$enum_name(self)
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
    pub fn new(cx: &'ast AstContext<'ast>, id: ItemId, vis: Visibility<'ast>, name: SymbolId) -> Self {
        Self { cx, id, vis, name }
    }
}

/// FIXME: Add function as  discussed in <https://github.com/rust-linting/design/issues/22>
/// this will require new driver callback functions
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Visibility<'ast> {
    _cx: &'ast AstContext<'ast>,
    _item_id: ItemId,
}

#[cfg(feature = "driver-api")]
impl<'ast> Visibility<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, item_id: ItemId) -> Self {
        Self {
            _cx: cx,
            _item_id: item_id,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Items based on traits
///////////////////////////////////////////////////////////////////////////////

pub trait FunctionItem<'ast>: ItemData<'ast> + FnDeclaration<'ast> {
    fn get_generics(&self);
}

/// This is a general trait that is implemented for all types of callable function
/// like items in Rust. This currently means: functions, methods and closures
///
/// Some getters will not be used for every function like item. For instance will
/// closures at the time of writing this always have the default [`Constness`] and
/// [`Abi`]
pub trait FnDeclaration<'ast>: Debug {
    /// Note that trait implementations are currently not allowed to be constant,
    /// this will therefore always return [`Constness::Default`] for trait functions.
    fn get_constness(&self) -> Constness;

    fn get_safety(&self) -> Safety;

    /// Note that trait implementations are currently not allowed to be async,
    /// this will therefore always return [`Asyncness::Default`] for trait functions.
    fn get_asyncness(&self) -> Asyncness;

    fn get_abi(&self) -> Abi;

    fn get_params(&self) -> &[&dyn FnParam<'ast>];

    /// The return type of a function without an explicit return type is the
    /// unit type `()`.
    fn get_return_ty(&self) -> &dyn TyOld<'ast>;

    /// This will always return a valid [`BodyId`] for functions, closures and
    /// methods. Trait functions can have a default body but they don't have to.
    fn get_body_id(&self) -> Option<BodyId>;
}

pub trait FnParam<'ast>: Debug {
    fn get_pattern(&self) -> &dyn Pattern<'ast>;

    fn get_span(&self) -> &Span<'ast>;

    fn get_ty(&self) -> &dyn TyOld<'ast>;
}

pub trait TypeAliasItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] of this type alias, this id is be different from
    /// the aliased type.
    fn get_ty_id(&self) -> TyIdOld;

    fn get_generics(&self);

    /// This can return `None` for [`TypeAliasItem`]s asscociated with a trait. For
    /// normal items this will always return `Some` at the time of writing this.
    fn get_aliased_ty(&self) -> Option<&dyn TyOld<'ast>>;
}

pub trait StructItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this struct.
    fn get_ty_id(&self) -> TyIdOld;

    fn get_kind(&self) -> AdtVariantData<'ast>;

    fn get_generics(&self);

    // FIXME: Add layout information
}

#[non_exhaustive]
#[derive(Debug)]
pub enum AdtVariantData<'ast> {
    /// A unit struct like:
    /// ```rs
    /// struct Name1;
    /// struct Name2 {};
    /// ```
    Unit,
    /// A tuple struct like:
    /// ```rs
    /// struct Name(u32, u64);
    /// ```
    /// This representation doesn't contain spans of each individual type, for diagnostics
    /// please span over the entire struct.
    Tuple(&'ast [&'ast dyn AdtField<'ast>]),
    /// A field struct like:ö
    /// ```rs
    /// struct Name {
    ///     field: u32,
    /// };
    /// ```
    /// Note: In the Rust Reference, this struct expression is called `StructExprStruct`
    /// here it has been called `Field`, to indicate that it uses fiels as opposed to the
    /// other kinds
    Field(&'ast [&'ast dyn AdtField<'ast>]),
}

/// A field in a struct of the form:
/// ```ignore
/// pub struct StructName {
///     #[some_attr]
///     pub name: Ty,
/// }
/// ```
///
/// For tuple structs the name will correspond with the field number.
pub trait AdtField<'ast>: Debug {
    fn get_attributes(&'ast self) -> &'ast dyn Attribute;

    /// This will return the span of the field, exclusing the field attributes.
    fn get_span(&'ast self) -> &'ast Span<'ast>;

    fn get_name(&'ast self) -> Symbol;

    fn get_ty(&'ast self) -> &'ast dyn TyOld<'ast>;
}

/// See: <https://doc.rust-lang.org/reference/items/enumerations.html>
pub trait EnumItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this struct.
    fn get_ty_id(&self) -> TyIdOld;

    fn get_variants(&self) -> &[&dyn EnumVariant<'ast>];

    fn get_generics(&self);

    // FIXME: Add layout information
}

pub trait EnumVariant<'ast>: Debug {
    /// This returns the discriminant expression if one has been defined.
    fn get_discriminant_expr(&self) -> Option<&dyn AnonConst<'ast>>;

    fn get_discriminant(&self) -> u128;

    fn get_name(&self) -> Symbol;

    fn get_variant_data(&self) -> AdtVariantData<'ast>;
}

/// An anonymous constant.
pub trait AnonConst<'ast>: Debug {
    fn get_ty(&self) -> &'ast dyn TyOld<'ast>;

    // FIXME: This should return a expression once they are implemented, it would
    // probably be good to have an additional `get_value_lit` that returns a literal,
    // if the value can be represented as one.
    fn get_value(&self);
}

pub trait UnionItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this union.
    fn get_ty_id(&self) -> TyIdOld;

    /// This will at the time of writitng this always return the [`AdtVariantData::Field`]
    /// variant. [`AdtVariantData`] is still used as a wrapper to support common util
    /// functionality and to possibly adapt [`AdtVariantData`] if the Rust standard expands.
    fn get_variant_data(&self) -> AdtVariantData<'ast>;

    // FIXME: Add layout information
}

pub trait TraitItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this trait.
    fn get_ty_id(&self) -> TyIdOld;

    fn get_safety(&self) -> Safety;

    fn get_generics(&self);

    fn get_super_traits(&self) -> &[&dyn TyOld<'ast>];

    /// This returns all associated items that are defined by this trait
    fn get_assoc_items(&self) -> &[AssocItem<'ast>];
}

#[non_exhaustive]
#[derive(Debug)]
pub enum AssocItem<'ast> {
    TypeAlias(&'ast dyn TypeAliasItem<'ast>),
    Const(&'ast ConstItem<'ast>),
    Function(&'ast dyn FunctionItem<'ast>),
}

pub trait ImplItem<'ast>: ItemData<'ast> {
    fn get_inner_attrs(&self); // FIXME: Add return type -> [&dyn Attribute<'ast>];

    fn get_safety(&self) -> Safety;

    fn get_polarity(&self) -> ImplPolarity;

    /// This will return `Some` if this is a trait implementation, otherwiese `None`.
    fn get_trait(&self) -> Option<&dyn TyOld<'ast>>;

    fn get_ty(&self) -> &dyn TyOld<'ast>;

    fn get_generics(&self);

    fn get_assoc_items(&self) -> &[AssocItem<'ast>];
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ImplPolarity {
    Positive,
    /// A negative implementation like:
    /// ```ignore
    /// unsafe impl !Send for ImplPolarity;
    /// //          ^
    /// ```
    Negative,
}

pub trait ExternBlockItem<'ast>: ItemData<'ast> {
    fn get_inner_attrs(&self); // FIXME: Add return type -> [&dyn Attribute<'ast>];

    fn get_safety(&self) -> Safety;

    fn get_abi(&self) -> Abi;

    fn get_external_items(&self) -> ExternalItems<'ast>;
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ExternalItems<'ast> {
    Static(&'ast StaticItem<'ast>),
    Function(&'ast dyn FunctionItem<'ast>),
}

#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UseKind {
    /// Single usages like `use foo::bar` a list of multiple usages like
    /// `use foo::{bar, baz}` will be desugured to `use foo::bar; use foo::baz;`
    Single,
    /// A glob import like `use foo::*`
    Glob,
}
