use std::fmt::Debug;

use super::{
    ty::{Mutability, Ty, TyId},
    Attribute, BodyId, Ident, SimplePath, Span, Spanned, Symbol,
};

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

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UseKind {
    /// Single usages like `use foo::bar` a list of multiple usages like
    /// `use foo::{bar, baz}` will be desugured to `use foo::bar; use foo::baz;`
    Single,
    /// A glob import like `use foo::*`
    Glob,
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

    fn get_kind(&'ast self) -> StructItemKind<'ast>;

    fn get_generics(&'ast self) -> &'ast dyn GenericDefs<'ast>;
}

#[non_exhaustive]
#[derive(Debug)]
pub enum StructItemKind<'ast> {
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
    Tuple(&'ast [&'ast dyn StructField<'ast>]),
    /// A field struct like:
    /// ```rs
    /// struct Name {
    ///     field: u32,
    /// };
    /// ```
    /// Note: In the Rust Reference, this struct expression is called `StructExprStruct`
    /// here it has been called `Field`, to indicate that it uses fiels as opposed to the
    /// other kinds
    Field(&'ast [&'ast dyn StructField<'ast>]),
}

/// A field in a struct of the from
/// ```ignore
/// pub struct StructName {
///     #[some_attr]
///     pub name: Ty,
/// }
/// ```
pub trait StructField<'ast>: Debug {
    fn get_attributes(&'ast self) -> &'ast dyn Attribute;

    /// This will return the span of the field, exclusing the field attributes.
    fn get_span(&'ast self) -> &'ast dyn Span<'ast>;

    fn get_visibility(&'ast self) -> VisibilityKind<'ast>;

    fn get_name(&'ast self) -> Symbol;

    fn get_ty(&'ast self) -> &'ast dyn Ty<'ast>;
}

/// The generic definitions belonging to an item
pub trait GenericDefs<'ast>: Debug {
    fn get_generics(&self) -> &'ast [&'ast dyn GenericParam<'ast>];
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParamId {
    krate: usize,
    index: usize,
}

#[cfg(feature = "driver-api")]
impl GenericParamId {
    #[must_use]
    pub fn new(krate: usize, index: usize) -> Self {
        Self { krate, index }
    }

    pub fn get_data(&self) -> (usize, usize) {
        (self.krate, self.index)
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

/// An anonymous constant.
pub trait AnonConst<'ast>: Debug {
    fn get_ty(&self) -> &'ast dyn Ty<'ast>;

    // FIXME: This should return a expression once they are implemented, it would
    // probably be good to have an additional `get_value_lit` that returns a literal,
    // if the value can be represented as one.
    fn get_value(&self);
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
