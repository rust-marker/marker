//! This module contains all representations of paths in the AST.
//!
//! See: <https://doc.rust-lang.org/stable/reference/paths.html>

// FIXME: It might be useful to not use a single path for everything, but instead
// split it up into an `ItemPath`, `GenericPath` etc. implementation.

use super::{GenericId, Ident, ItemId, VarId, VariantId};
use crate::{
    ast::{generic::GenericArgs, ty::TyKind},
    ffi::{FfiOption, FfiSlice},
};

/// [`AstPath`]s are used to identify items. A qualified path (`QPath`) can be
/// used in expressions and types to identify associated items on types. For
/// traits it's additionally possible to specify the type that should be used
/// as `Self`. This is sometimes needed to disambiguate an item, if it exists
/// both as an associated item on a type, and as an associated item on traits,
/// that this type implements.
///
/// In the following example, the `Item` has two implementations of the associated
/// `foo()` function. One is provided by the `impl` block and the other one by the
/// `Foo` trait. When calling `Item::foo()` the `impl` block implementation will
/// targeted. To access the trait function, the path has to be specified, with `Item`
/// declared as the `Self` type.
///
/// ```
/// // Item
/// struct Item;
/// impl Item {
///     fn foo() {
///         println!("foo() from Item")
///     }
/// }
///
/// // trait
/// trait Foo {
///     fn foo();
/// }
/// impl Foo for Item {
///     fn foo() {
///         println!("foo() from Foo trait");
///     }
/// }
///
/// // Calls the `foo()` method of `impl Item`
/// Item::foo(); // -> "foo() from Item"
///
/// // Calls the `foo()` method of `trait Foo` with `Item` as the `Self` type
/// <Item as Foo>::foo(); // -> "foo() from Foo trait"
/// ```
///
/// This representation can also be used to reference non-associated items, to
/// make it more flexible. For these items, the path type will be [`None`]. The
/// target can be resolved via the [`resolve()`](AstQPath::resolve) method.
/// Alternatively, the [`AstPath`] representation can be accessed via
/// [`as_path_lossy()`](AstQPath::as_path_lossy) or the [`TryInto`] implementation.
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstQPath<'ast> {
    self_ty: FfiOption<TyKind<'ast>>,
    path_ty: FfiOption<TyKind<'ast>>,
    path: AstPath<'ast>,
    target: AstPathTarget,
}

impl<'ast> AstQPath<'ast> {
    /// This method will return [`Some`], if the path has a specified `Self`
    /// type. The main type for type relative paths is provided by
    /// [`path_ty()`][`AstQPath::path_ty()`].
    ///
    /// ```
    /// # let _: Vec<i32> =
    ///     <Vec<_> as Default>::default();
    /// //   ^^^^^^ The specified `Self` type `Vec<_>`
    /// ```
    ///
    /// The [`AstQPath`] description contains more details, when this might
    /// be necessary.
    pub fn self_ty(&self) -> Option<TyKind<'ast>> {
        self.self_ty.copy()
    }

    /// This method will return [`Some`], if the path is type relative.
    ///
    /// ```
    /// # let _: Vec<i32> =
    ///     <Vec<_> as Default>::default();
    /// //             ^^^^^^^ The path is relative to the `Default` trait
    /// ```
    ///
    /// The optional `Self` type can be accessed via [`self_ty`](AstQPath::self_ty()).
    pub fn path_ty(&self) -> Option<TyKind<'ast>> {
        self.path_ty.copy()
    }

    /// This returns a [`AstPath`] of the referenced item. For type relative
    /// paths, this will include the type itself, if the type can be expressed
    /// as a [`AstPathSegment`]. For some types l,ike slices, this is not possible.
    /// The type has to be retrieved from [`path_ty()`][`AstQPath::path_ty()`].
    ///
    /// ### Examples:
    ///
    /// ```
    /// let _: Vec<i32> = Vec::default();
    /// // AstPath: `Vec::default`
    ///
    /// let _: Vec<i32> = Default::default();
    /// // AstPath: `Default::default`
    ///
    /// let _: Vec<i32> = <Vec<_> as Default>::default();
    /// // AstPath: `Default::default`
    ///
    ///  let _ = [0_u8].is_ascii();
    /// // AstPath: `is_ascii`
    /// ```
    ///
    /// ### Warning
    ///
    /// The method is lossy, as the optional `Self` type isn't included in this
    /// path. The path type might also be missing, if it can't be represented as
    /// a [`AstPathSegment`]. The conversion is lossless, if both types are none
    /// or if the path type can be represented. To resolve a qualified path
    /// [`resolve()`](Self::resolve()) should be used.
    ///
    /// Omitting the `Self` type can be useful, in cases, where access to associated
    /// trait items should be analyzed, regardless of potential `Self` types.
    /// Alternatively, [`segments()`](AstQPath::segments()) can be used to access the
    /// segments directly.
    pub fn as_path_lossy(&self) -> &AstPath<'ast> {
        &self.path
    }

    /// This returns the [`AstPathSegment`]s of the path. For type relative
    /// paths, this will include the type itself. The optional `Self` type
    /// isn't represented in these segments. These segments are identical with
    /// the segments provided by the path of
    /// [`as_path_lossy()`](AstQPath::as_path_lossy()). The documentation
    /// of that function contains more details.
    pub fn segments(&self) -> &[AstPathSegment<'ast>] {
        self.path.segments()
    }

    /// This function resolves the target of this path.
    pub fn resolve(&self) -> AstPathTarget {
        // For rust-analyzer or future drivers, it might make sense to return
        // `Option<AstPathTarget>` instead, as the path might be dead,
        // when a lint crate calls this function. However, I have the feeling
        // that this would make the API less ergonomic. The `AstContext` will
        // already need to handle these cases explicitly. Currently, a user can
        // get a resolved id from the target, but the resolution of the ID, by
        // the `AstContext`, might fail. The outcome is the same, but all
        // "failable" resolution will be grouped in the `AstContext`
        self.target
    }

    /// This returns the [`GenericArgs`] specified on the last segment of the path.
    /// This is especially useful, for paths pointing to types or functions. For
    /// example, the `u32` of the path `Vec<u32>`, is stored in the [`GenericArgs`]
    /// as a type parameter.
    pub fn generics(&self) -> &GenericArgs<'ast> {
        self.path.generics()
    }
}

impl<'a, 'ast> TryFrom<&'a AstQPath<'ast>> for &'a AstPath<'ast> {
    type Error = ();

    fn try_from(value: &'a AstQPath<'ast>) -> Result<Self, Self::Error> {
        fn is_segment_representable(ty: Option<TyKind<'_>>) -> bool {
            if let Some(ty) = ty {
                ty.is_primitive_ty()
                    || matches!(ty, TyKind::Path(path_ty) if is_segment_representable(path_ty.path().path_ty()))
            } else {
                true
            }
        }
        if value.self_ty.is_some() && is_segment_representable(value.path_ty.copy()) {
            Err(())
        } else {
            Ok(&value.path)
        }
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> AstQPath<'ast> {
    pub fn new(
        self_ty: Option<TyKind<'ast>>,
        path_ty: Option<TyKind<'ast>>,
        path: AstPath<'ast>,
        target: AstPathTarget,
    ) -> Self {
        Self {
            self_ty: self_ty.into(),
            path_ty: path_ty.into(),
            path,
            target,
        }
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AstPathTarget {
    /// The `Self` type, the [`ItemId`] points to the item,
    /// that the `Self` originates from. This will usually be an
    /// [`ImplItem`](crate::ast::item::ImplItem) or
    /// [`TraitItem`](crate::ast::item::TraitItem).
    SelfTy(ItemId),
    /// The path points to an item, identified by the [`ItemId`]. For example
    /// [`ConstItem`](crate::ast::item::ConstItem),
    /// [`StaticItem`](crate::ast::item::ImplItem),
    /// [`FnItem`](crate::ast::item::FnItem).
    Item(ItemId),
    /// The path target is a variant from an enum, identified by the [`VariantId`]
    Variant(VariantId),
    /// The path target is a local variable, identified by the [`VarId`].
    Var(VarId),
    /// The path target is a generic type, identified by the [`GenericId`].
    Generic(GenericId),
    /// The target can't be resolved in the current context. This can happen
    /// for paths in generic bounds.
    Unresolved,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstPath<'ast> {
    // FIXME: Add optional target ID for values, lifetimes, etc that is faster to compare
    segments: FfiSlice<'ast, AstPathSegment<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstPath<'ast> {
    pub fn new(segments: &'ast [AstPathSegment<'ast>]) -> Self {
        debug_assert!(!segments.is_empty());
        Self {
            segments: segments.into(),
        }
    }
}

impl<'ast> AstPath<'ast> {
    pub fn segments(&self) -> &[AstPathSegment<'ast>] {
        self.segments.get()
    }

    /// This returns the [`GenericArgs`] specified on the last segment of the path.
    /// This is especially useful, for paths pointing to types or functions. For
    /// example, the `u32` of the path `Vec<u32>`, is stored in the [`GenericArgs`]
    /// as a type parameter.
    pub fn generics(&self) -> &GenericArgs<'ast> {
        self.segments
            .get()
            .last()
            .expect("a path always has at least one segment")
            .generics()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub struct AstPathSegment<'ast> {
    ident: Ident<'ast>,
    generics: GenericArgs<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstPathSegment<'ast> {
    pub fn new(ident: Ident<'ast>, generics: GenericArgs<'ast>) -> Self {
        Self { ident, generics }
    }
}

impl<'ast> AstPathSegment<'ast> {
    pub fn ident(&self) -> &Ident<'ast> {
        &self.ident
    }

    pub fn generics(&self) -> &GenericArgs<'ast> {
        &self.generics
    }
}
