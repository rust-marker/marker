use std::fmt::Debug;

mod extern_crate_item;
pub use self::extern_crate_item::ExternCrateItem;
mod mod_item;
pub use self::mod_item::ModItem;
mod static_item;
pub use self::static_item::StaticItem;
mod use_decl_item;
pub use self::use_decl_item::UseDeclItem;

use super::{
    ty::{Ty, TyId},
    Abi, Asyncness, Attribute, BodyId, Constness, CrateId, Path, Pattern, Safety, Span, Symbol,
};

/// Every item has an ID that can be used to retive that item or compair it to
/// another id. The ID's can change in between linting sessions.
///
/// The interal representation is currently based on rustc's `DefId`. This might
/// change in the future. The struct will continue to provide the current trait
/// implementations.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    krate: CrateId,
    index: u32,
}

#[cfg(feature = "driver-api")]
impl ItemId {
    pub fn new(krate: CrateId, index: u32) -> Self {
        Self { krate, index }
    }
}

pub trait ItemData<'ast>: Debug {
    /// Returns the [`ItemId`] of this item. This is a unique identifier used for comparison
    /// and to request items from the [`AstContext`][`crate::context::AstContext`].
    fn get_id(&self) -> ItemId;

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    fn get_span(&self) -> &'ast dyn Span<'ast>;

    /// The visibility of this item.
    fn get_vis(&self) -> &Visibility<'ast>;

    /// This function can return `None` if the item was generated and has no real name
    fn get_name(&self) -> Option<Symbol>;

    /// This returns this [`ItemData`] instance as a [`ItemType`]. This can be useful for
    /// functions that take [`ItemType`] as a parameter. For general function calls it's better
    /// to call them directoly on the item, instead of converting it to a [`ItemType`] first.
    fn as_item(&'ast self) -> ItemType<'ast>;

    fn get_attrs(&self); // FIXME: Add return type: -> &'ast [&'ast dyn Attribute<'ast>];
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
    impl_item_type_fn!(get_id() -> ItemId);
    impl_item_type_fn!(get_span() -> &'ast dyn Span<'ast>);
    impl_item_type_fn!(get_vis() -> &Visibility<'ast>);
    impl_item_type_fn!(get_name() -> Option<Symbol>);
    impl_item_type_fn!(get_attrs() -> ());
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
    id: ItemId,
    span: &'ast dyn Span<'ast>,
    vis: Visibility<'ast>,
    name: Option<Symbol>,
}

macro_rules! impl_item_data {
    ($self_name:ident, $enum_name:ident) => {
        impl<'ast> super::ItemData<'ast> for $self_name<'ast> {
            fn get_id(&self) -> crate::ast::item::ItemId {
                self.data.id
            }

            fn get_span(&self) -> &'ast dyn crate::ast::Span<'ast> {
                self.data.span
            }

            fn get_vis(&self) -> &crate::ast::item::Visibility<'ast> {
                &self.data.vis
            }

            fn get_name(&self) -> Option<crate::ast::Symbol> {
                self.data.name
            }

            fn as_item(&'ast self) -> crate::ast::item::ItemType<'ast> {
                $crate::ast::item::ItemType::$enum_name(self)
            }

            fn get_attrs(&self) {
                todo!()
            }
        }
    };
}

use impl_item_data;

#[cfg(feature = "driver-api")]
impl<'ast> CommonItemData<'ast> {
    pub fn new(id: ItemId, span: &'ast dyn Span<'ast>, vis: Visibility<'ast>, name: Option<Symbol>) -> Self {
        Self { id, span, vis, name }
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

    fn get_span(&self) -> &dyn Span<'ast>;

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
    fn get_span(&'ast self) -> &'ast dyn Span<'ast>;

    fn get_visibility(&'ast self) -> Visibility<'ast>;

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
pub enum Visibility<'ast> {
    Pub,
    /// Visible in the current module, equivialent to `pub(in self)` or no visibility
    PubSelf,
    PubCrate,
    PubPath(&'ast Path<'ast>),
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
    fn get_span(&self) -> &'ast dyn Span<'ast>;

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
