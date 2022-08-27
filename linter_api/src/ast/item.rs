use std::fmt::Debug;

mod extern_crate_item;
use crate::context::AstContext;

pub use self::extern_crate_item::ExternCrateItem;
mod mod_item;
pub use self::mod_item::ModItem;
mod static_item;
pub use self::static_item::StaticItem;
mod use_decl_item;
pub use self::use_decl_item::UseDeclItem;

use super::{
    ty::{Ty, TyId},
    Abi, Asyncness, Attribute, BodyId, Constness, ItemId, ItemPath, Pattern, Safety, Span, Symbol, SymbolId,
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
    fn name(&self) -> Option<SymbolId>;

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
    Const(&'ast dyn ConstItem<'ast>),
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
    impl_item_type_fn!(name() -> Option<SymbolId>);
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
                self.data.cx.get_span(&crate::ast::SpanOwner::Item(self.data.id))
            }

            fn visibility(&self) -> &crate::ast::item::Visibility<'ast> {
                &self.data.vis
            }

            fn name(&self) -> Option<crate::ast::SymbolId> {
                Some(self.data.name)
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
#[derive(Debug)]
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

/// A constant item like
/// ```rs
/// const CONST_ITEM: u32 = 0xcafe;
/// // `get_name()` -> `CONST_ITEM`
/// // `get_ty()` -> _Ty of u32_
/// // `get_body_id()` -> _BodyId of `0xcafe`_
/// ```
pub trait ConstItem<'ast>: ItemData<'ast> {
    fn get_ty(&'ast self) -> &'ast dyn Ty<'ast>;

    /// The [`BodyId`] of the initialization body.
    ///
    /// This can return `None` for [`ConstItem`]s asscociated with a trait. For
    /// normal items this will always return `Some` at the time of writing this.
    fn get_body_id(&self) -> Option<BodyId>;
}

pub trait FunctionItem<'ast>: ItemData<'ast> + FnDeclaration<'ast> {
    fn get_generics(&self) -> &dyn GenericDefs<'ast>;
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
    fn get_return_ty(&self) -> &dyn Ty<'ast>;

    /// This will always return a valid [`BodyId`] for functions, closures and
    /// methods. Trait functions can have a default body but they don't have to.
    fn get_body_id(&self) -> Option<BodyId>;
}

pub trait FnParam<'ast>: Debug {
    fn get_pattern(&self) -> &dyn Pattern<'ast>;

    fn get_span(&self) -> &Span<'ast>;

    fn get_ty(&self) -> &dyn Ty<'ast>;
}

pub trait TypeAliasItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] of this type alias, this id is be different from
    /// the aliased type.
    fn get_ty_id(&self) -> TyId;

    fn get_generics(&self) -> &dyn GenericDefs<'ast>;

    /// This can return `None` for [`TypeAliasItem`]s asscociated with a trait. For
    /// normal items this will always return `Some` at the time of writing this.
    fn get_aliased_ty(&self) -> Option<&dyn Ty<'ast>>;
}

pub trait StructItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this struct.
    fn get_ty_id(&self) -> TyId;

    fn get_kind(&self) -> AdtVariantData<'ast>;

    fn get_generics(&self) -> &'ast dyn GenericDefs<'ast>;

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

    fn get_visibility(&'ast self) -> VisibilityOld<'ast>;

    fn get_name(&'ast self) -> Symbol;

    fn get_ty(&'ast self) -> &'ast dyn Ty<'ast>;
}

/// See: <https://doc.rust-lang.org/reference/items/enumerations.html>
pub trait EnumItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this struct.
    fn get_ty_id(&self) -> TyId;

    fn get_variants(&self) -> &[&dyn EnumVariant<'ast>];

    fn get_generics(&self) -> &'ast dyn GenericDefs<'ast>;

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
    fn get_ty(&self) -> &'ast dyn Ty<'ast>;

    // FIXME: This should return a expression once they are implemented, it would
    // probably be good to have an additional `get_value_lit` that returns a literal,
    // if the value can be represented as one.
    fn get_value(&self);
}

pub trait UnionItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this union.
    fn get_ty_id(&self) -> TyId;

    /// This will at the time of writitng this always return the [`AdtVariantData::Field`]
    /// variant. [`AdtVariantData`] is still used as a wrapper to support common util
    /// functionality and to possibly adapt [`AdtVariantData`] if the Rust standard expands.
    fn get_variant_data(&self) -> AdtVariantData<'ast>;

    // FIXME: Add layout information
}

pub trait TraitItem<'ast>: ItemData<'ast> {
    /// Returns the [`TyId`] for this trait.
    fn get_ty_id(&self) -> TyId;

    fn get_safety(&self) -> Safety;

    fn get_generics(&self) -> &dyn GenericDefs<'ast>;

    fn get_super_traits(&self) -> &[&dyn Ty<'ast>];

    /// This returns all associated items that are defined by this trait
    fn get_assoc_items(&self) -> &[AssocItem<'ast>];
}

#[non_exhaustive]
#[derive(Debug)]
pub enum AssocItem<'ast> {
    TypeAlias(&'ast dyn TypeAliasItem<'ast>),
    Const(&'ast dyn ConstItem<'ast>),
    Function(&'ast dyn FunctionItem<'ast>),
}

pub trait ImplItem<'ast>: ItemData<'ast> {
    fn get_inner_attrs(&self); // FIXME: Add return type -> [&dyn Attribute<'ast>];

    fn get_safety(&self) -> Safety;

    fn get_polarity(&self) -> ImplPolarity;

    /// This will return `Some` if this is a trait implementation, otherwiese `None`.
    fn get_trait(&self) -> Option<&dyn Ty<'ast>>;

    fn get_ty(&self) -> &dyn Ty<'ast>;

    fn get_generics(&self) -> &dyn GenericDefs<'ast>;

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

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UseKind {
    /// Single usages like `use foo::bar` a list of multiple usages like
    /// `use foo::{bar, baz}` will be desugured to `use foo::bar; use foo::baz;`
    Single,
    /// A glob import like `use foo::*`
    Glob,
}

/// The visibility of items.
///
/// See: <https://doc.rust-lang.org/reference/visibility-and-privacy.html>
#[non_exhaustive]
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum VisibilityOld<'ast> {
    Pub,
    /// Visible in the current module, equivialent to `pub(in self)` or no visibility
    PubSelf,
    PubCrate,
    PubPath(&'ast ItemPath<'ast>),
    PubSuper,
    None,
}

/// The generic definitions belonging to an item
pub trait GenericDefs<'ast>: Debug {
    fn get_generics(&self) -> &'ast [&'ast dyn GenericParam<'ast>];

    /// This function returns all bounds set for the given item. The bounds are a
    /// combination of direct bounds (`<T: Debug>`) and `where` conditions (`where T: Debug`)
    fn get_bounds(&self); // FIXME: Add return type
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParamId {
    owner: ItemId,
    index: usize,
}

#[cfg(feature = "driver-api")]
impl GenericParamId {
    #[must_use]
    pub fn new(owner: ItemId, index: usize) -> Self {
        Self { owner, index }
    }

    pub fn get_data(&self) -> (ItemId, usize) {
        (self.owner, self.index)
    }
}

/// A generic parameter for a function or struct.
pub trait GenericParam<'ast>: Debug {
    fn get_id(&self) -> GenericParamId;

    /// This returns the span of generic identifier.
    fn get_span(&self) -> &'ast Span<'ast>;

    /// This returns the name of the generic, this can return `None` for unnamed
    /// or implicit generics. For lifetimes this will include the leading apostrophe.
    ///
    /// Examples: `T`, `'ast`
    fn get_name(&self) -> Option<Symbol>;

    fn get_kind(&self) -> GenericKind<'ast>;
}

#[non_exhaustive]
#[derive(Debug)]
pub enum GenericKind<'ast> {
    Lifetime,
    Type {
        default: Option<&'ast dyn Ty<'ast>>,
    },
    Const {
        ty: &'ast dyn Ty<'ast>,
        default: Option<&'ast dyn AnonConst<'ast>>,
    },
}

/// This represents a single bound for a given generic. Several bounds will be split up
/// into multiple predicates:
///
/// | Rust                   | Simplified Representation                                  |
/// | ---------------------- | ---------------------------------------------------------  |
/// | `'x: 'a + 'b + 'c`     | ``[Outlives('x: 'a), Outlives('x: 'b), Outlives('x: 'c)]`` |
/// | `T: Debug + 'a`        | ``[TraitBound(`T: Debug`), LifetimeBound(`T: 'a`)]``       |
///
/// FIXME: This is still missing a representation for predicates with for lifetimes like:
/// `for<'a> T: 'a`
#[non_exhaustive]
#[derive(Debug)]
pub enum GenericPredicate<'ast> {
    /// A outlive bound like:
    /// * `'a: 'x`
    Outlives {
        lifetime: GenericParamId,
        outlived_by: GenericParamId,
    },
    /// A trait bound like:
    /// * `T: Debug`
    /// * `T: ?Sized`
    /// * `I::Item: Copy`
    TraitBound {
        ty: &'ast dyn Ty<'ast>,
        bound: TyId,
        modifier: TraitBoundModifier,
    },
    /// A type lifetime bound like: `T: 'a`
    LifetimeBound {
        ty: &'ast dyn Ty<'ast>,
        outlived_by: GenericParamId,
    },
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TraitBoundModifier {
    /// A trait like: `T: Debug`
    None,
    /// An optional trait like: `T: ?Sized`
    Maybe,
}
