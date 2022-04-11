use std::fmt::Debug;

use super::{
    ty::{Mutability, Ty, TyId},
    Abi, Asyncness, Attribute, BodyId, Constness, Ident, Pattern, Safety, SimplePath, Span, Spanned, Symbol,
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

pub trait ItemData<'ast>: Debug {
    /// Returns the [`ItemId`] of this item. This is a unique identifier used for comparison
    /// and to request items from the [`LintPassContext`][`crate::context::LintPassContext`].
    fn get_id(&self) -> ItemId;

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    fn get_span(&self) -> &'ast dyn Span<'ast>;

    /// The visibility of this item.
    fn get_vis(&self) -> &'ast Visibility<'ast>;

    /// This function can return `None` if the item was generated and has no real name
    fn get_name(&self) -> Option<Symbol>;

    /// This returns this [`ItemData`] instance as a [`ItemType`]. This can be usefull for
    /// functions that take [`ItemType`] as a parameter. For general function calls it's better
    /// to call them directoly on the item, instead of converting it to a [`ItemType`] first.
    fn as_item(&self) -> ItemType<'ast>;

    fn get_attrs(&self); // FIXME: Add return type: -> &'ast [&'ast dyn Attribute<'ast>];
}

#[non_exhaustive]
#[derive(Debug, Clone)]
// TODO: Fix `ItemItem` names
pub enum ItemType<'ast> {
    Mod(&'ast dyn ModItem<'ast>),
    ExternCrate(&'ast dyn ExternCrateItem<'ast>),
    UseDeclaration(&'ast dyn UseDeclarationItem<'ast>),
    StaticItem(&'ast dyn StaticItemItem<'ast>),
    ConstItem(&'ast dyn ConstItemItem<'ast>),
    Function(&'ast dyn FunctionItem<'ast>),
    TypeAlias(&'ast dyn TypeAliasItem<'ast>),
    Struct(&'ast dyn StructItemItem<'ast>),
    Enum(&'ast dyn EnumItem<'ast>),
    Union(&'ast dyn UnionItem<'ast>),
    Trait(&'ast dyn TraitItem<'ast>),
    Implementation(&'ast dyn ImplementationItem<'ast>),
    ExternBlock(&'ast dyn ExternBlockItem<'ast>),
}

impl<'ast> ItemType<'ast> {
    impl_item_type_fn!(get_id() -> ItemId);
    impl_item_type_fn!(get_span() -> &'ast dyn Span<'ast>);
    impl_item_type_fn!(get_vis() -> &'ast Visibility<'ast>);
    impl_item_type_fn!(get_name() -> Option<Symbol>);
    impl_item_type_fn!(get_attrs() -> ());
}

/// Until [trait upcasting](https://github.com/rust-lang/rust/issues/65991) has been implemented
/// and stabalized we need this to call [`ItemData`] functions for [`ItemType`].
macro_rules! impl_item_type_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_item_type_fn!($method() -> $return_ty,
            Mod, ExternCrate, UseDeclaration, StaticItem, ConstItem, Function,
            TypeAlias, Struct, Enum, Union, Trait, Implementation, ExternBlock
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

pub(crate) use impl_item_type_fn;

pub trait ModItem<'ast>: ItemData<'ast> {
    fn get_inner_attrs(&self); // FIXME: Add return type -> [&dyn Attribute<'ast>];

    fn get_items(&self) -> &[ItemType<'ast>];
}

/// ```ignore
/// extern crate std;
/// // `get_name()`          -> "std"
/// // `get_original_name()` -> "std"
/// extern crate std as ruststd;
/// // `get_name()`          -> "ruststd"
/// // `get_original_name()` -> "std"
/// ```
pub trait ExternCrateItem<'ast>: ItemData<'ast> {
    /// This will return the original name of external crate. This will only differ
    /// with [`ItemData::get_name`] if the user has declared an alias with as.
    fn get_original_name(&self) -> Symbol;
}

/// ```ignore
/// pub use foo::bar::*;
/// // `get_name()`     -> `None`
/// // `get_path()`     -> `foo::bar::*`
/// // `get_use_kind()` -> `Glob`
/// pub use foo::bar;
/// // `get_name()`     -> `Some(bar)`
/// // `get_path()`     -> `foo::bar`
/// // `get_use_kind()` -> `Single`
/// pub use foo::bar as baz;
/// // `get_name()`     -> `Some(baz)`
/// // `get_path()`     -> `foo::bar`
/// // `get_use_kind()` -> `Single`
/// ```
pub trait UseDeclarationItem<'ast>: ItemData<'ast> {
    /// Returns the path of this `use` item. For blob imports the `*` will
    /// be included in the simple path.
    fn get_path(&self) -> &dyn SimplePath<'ast>;

    fn get_use_kind(&self) -> UseKind;
}

/// ```ignore
/// static mut LEVELS: u32 = 0;
/// // `get_name()` -> `LEVELS`
/// // `get_mutability()` -> _Mutable_
/// // `get_ty()` -> _Ty of u32_
/// // `get_body_id()` -> _BodyId of `0`_
/// ```
pub trait StaticItemItem<'ast>: ItemData<'ast> {
    /// The mutability of this item
    fn get_mutability(&self) -> Mutability;

    /// The defined type of this static item
    fn get_ty(&'ast self) -> &'ast dyn Ty<'ast>;

    /// This returns the [`BodyId`] of the initialization body.
    fn get_body_id(&self) -> BodyId;
}

/// A constant item like
/// ```rs
/// const CONST_ITEM: u32 = 0xcafe;
/// // `get_name()` -> `CONST_ITEM`
/// // `get_ty()` -> _Ty of u32_
/// // `get_body_id()` -> _BodyId of `0xcafe`_
/// ```
pub trait ConstItemItem<'ast>: ItemData<'ast> {
    fn get_ty(&'ast self) -> &'ast dyn Ty<'ast>;

    /// The [`BodyId`] of the initialization body.
    ///
    /// This can return `None` for [`ConstItemItem`]s asscociated with a trait. For
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

pub trait StructItemItem<'ast>: ItemData<'ast> {
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

    fn get_visibility(&'ast self) -> VisibilityKind<'ast>;

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
    Const(&'ast dyn ConstItemItem<'ast>),
    Function(&'ast dyn FunctionItem<'ast>),
}

pub trait ImplementationItem<'ast>: ItemData<'ast> {
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
    Static(&'ast dyn StaticItemItem<'ast>),
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
pub enum VisibilityKind<'ast> {
    /// Visible in the current module, equivialent to `pub(in self)` or no visibility
    PubSelf,
    PubCrate,
    /// FIXME: Add a path value to this
    PubPath(&'ast dyn SimplePath<'ast>),
    PubSuper,
}

// ===========================================================================
// OLD ITEMS
// ===========================================================================

pub trait Item<'ast>: Debug {
    fn get_id(&self) -> ItemId;

    fn get_span(&'ast self) -> &'ast dyn Span<'ast>;

    fn get_vis(&self) -> &'ast Visibility<'ast>;

    /// This function can return `None` if the item was generated and has no real name
    fn get_ident(&'ast self) -> Option<&'ast Ident<'ast>>;

    fn get_kind(&'ast self) -> ItemKind<'ast>;

    fn get_attrs(&'ast self) -> &'ast [&dyn Attribute<'ast>];
}

pub type Visibility<'ast> = Spanned<'ast, VisibilityKind<'ast>>;

#[non_exhaustive]
#[derive(Debug)]
pub enum ItemKind<'ast> {
    Mod(&'ast dyn ModuleItem<'ast>),
    /// An `extern crate` item, with an optional *original* create name. The given
    /// and used name is the identifier of the [`Item`].
    ExternCrate(Option<Symbol>),
    UseDeclaration(&'ast dyn SimplePath<'ast>, UseKind),
    StaticItem(&'ast dyn StaticItem<'ast>),
    ConstItem(&'ast dyn ConstItem<'ast>),
    Function,
    TypeAlias,
    Struct(&'ast dyn StructItem<'ast>),
    Enumeration,
    Union,
    Trait,
    Implementation,
    ExternBlock,
}

pub trait ModuleItem<'ast>: Debug {
    fn get_inner_attrs(&'ast self) -> [&dyn Attribute<'ast>];

    fn get_items(&'ast self) -> [&dyn Item<'ast>];
}

/// A static item like
/// ```rs
/// pub static STATIC_ITEM: u32 = 18;
/// ```
pub trait StaticItem<'ast>: Debug {
    fn get_type(&'ast self) -> &'ast dyn Ty<'ast>;

    fn get_mutability(&self) -> Mutability;

    fn get_body_id(&self) -> BodyId;
}

/// A constant item like
/// ```rs
/// const CONST_ITEM: u32 = 0xcafe;
/// ```
pub trait ConstItem<'ast>: Debug {
    fn get_type(&'ast self) -> &'ast dyn Ty<'ast>;

    fn get_body_id(&self) -> BodyId;
}

/// A struct item like:
///
/// ```rs
/// pub struct ExampleOne;
///
/// pub struct ExampleTwo(u32, &'static str);
///
/// pub struct ExampleThree<T> {
///     field: T,
/// }
/// ```
pub trait StructItem<'ast>: Debug {
    /// Returns the [`TyId`] for this struct.
    fn get_ty_id(&self) -> TyId;

    fn get_kind(&'ast self) -> AdtVariantData<'ast>;

    fn get_generics(&'ast self) -> &'ast dyn GenericDefs<'ast>;

    // FIXME: Provide layout information for this ADT
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

    /// This returns the name of the generic, this can retrun `None` for unnamed
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
