//! This module contains all representations of paths in the AST.
//!
//! See: <https://doc.rust-lang.org/stable/reference/paths.html>

// FIXME: It might be useful to not use a single path for everything, but instead
// split it up into an `ItemPath`, `GenericPath` etc. implementation.

use super::{Ident, ItemId, VarId};
use crate::{
    ast::{generic::GenericArgs, ty::TyKind},
    ffi::{FfiOption, FfiSlice},
};

/// A path identifying a unique item. The path might be type relative
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct QualifiedAstPath<'ast> {
    self_ty: FfiOption<TyKind<'ast>>,
    path_ty: FfiOption<TyKind<'ast>>,
    path: AstPath<'ast>,
    target: QualifiedPathTarget,
}

impl<'ast> QualifiedAstPath<'ast> {
    /// Qualified paths can be type relative. Additionally, a type can be specified
    /// that should be used as `Self`. The specification of the `Self` type is
    /// sometimes needed to resolve the right item, when accessing trait implementations.
    ///
    /// In the following example, the `Item` has two implementations of the
    /// `foo()` function. One is provided by the `impl` block and the other one
    /// by the `Foo` trait. When calling `Item::foo()` the `impl` block implementation
    /// will targeted. To access the trait function, the path to it has to be
    /// specified, with `Item` declared as the `Self` type.
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
    ///         println!("foo() from Trait");
    ///     }
    /// }
    ///
    /// // Calls the `foo()` method of `impl Item`
    /// Item::foo();
    /// // Calls the `foo()` method of `trait Foo` with `Item` as `Self`
    /// <Item as Foo>::foo();
    /// ```
    ///
    /// This function will only return a `Some`, if a `Self` type is specified.
    /// The type for a normal type relative path is provided by
    /// [`path_ty()`][`QualifiedAstPath::path_ty()`].
    pub fn self_ty(&self) -> Option<TyKind<'ast>> {
        self.self_ty.copy()
    }

    /// This returns the type of the path, if the path is type relative, `None`
    /// otherwise. In some cases, the path might include the specification of a
    /// `Self` type, to resolve the correct item. See
    /// [`self_ty()`][`QualifiedAstPath::self_ty()`] for a detailed explanation
    /// and example.
    pub fn path_ty(&self) -> Option<TyKind<'ast>> {
        self.path_ty.copy()
    }

    /// This returns a [`AstPath`] of the referenced item. For type relative
    /// paths, this will include the type itself. For example:
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
    /// ```
    ///
    /// The method is lossy. as the optional `Self` type isn't included in this
    /// path. To resolve the target, the qualified path should be used. See
    /// [`self_ty()`][`QualifiedAstPath::self_ty()`] for a detailed explanation
    /// of the `Self` type.
    pub fn to_path_lossy(&self) -> &AstPath<'ast> {
        &self.path
    }

    /// This returns the [`AstPathSegment`]s of the path. For type relative
    /// paths, this will include the type itself. The optional `Self` type
    /// isn't represented in these segments. These segments are identical with
    /// the segments provided by the path of.
    /// [`to_path_lossy()`](QualifiedAstPath::to_path_lossy). The documentation
    /// of that function contains more details.
    pub fn segments(&self) -> &[AstPathSegment<'ast>] {
        self.path.segments()
    }

    /// This function resolves the target of this path.
    pub fn resolve(&self) -> QualifiedPathTarget {
        // For rust-analyzer or future drivers, it might make sense to return
        // `Option<QualifiedPathTarget>` instead, as the path might be dead,
        // when a lint crate calls this function. However, I have the feeling
        // that this would make the API less ergonomic. The `AstContext` will
        // already need to handle these cases explicitly. Currently, a user can
        // get a resolved id from the target, but the resolution of the ID, by
        // the `AstContext`, might fail. The outcome is the same, but all
        // "failable" resolution will be grouped in the `AstContext`
        self.target
    }
}

impl<'a, 'ast> TryFrom<&'a QualifiedAstPath<'ast>> for &'a AstPath<'ast> {
    type Error = ();

    fn try_from(value: &'a QualifiedAstPath<'ast>) -> Result<Self, Self::Error> {
        if value.self_ty.is_some() {
            Err(())
        } else {
            Ok(&value.path)
        }
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum QualifiedPathTarget {
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
    /// The path target is a local variable, identified by the [`VarId`].
    Var(VarId),
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
        Self {
            segments: segments.into(),
        }
    }
}

impl<'ast> AstPath<'ast> {
    pub fn segments(&self) -> &[AstPathSegment<'ast>] {
        self.segments.get()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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
