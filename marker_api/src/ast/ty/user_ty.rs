use std::marker::PhantomData;

use crate::ast::{generic::SemGenericArgs, AstQPath, GenericId, ItemId, TyDefId};

use super::CommonSynTyData;

/// A type identified via a [`AstQPath`]. The kind and definition can be
/// accessed via the ID returned by [`AstQPath::resolve()`].
///
/// A path type is used for:
/// * [Generic types](https://doc.rust-lang.org/reference/items/generics.html#generic-parameters)
/// * [Type aliases](https://doc.rust-lang.org/reference/items/type-aliases.html#type-aliases)
/// * [`Self` types](<https://doc.rust-lang.org/stable/std/keyword.SelfTy.html>)
/// * User defined types like [Structs](https://doc.rust-lang.org/reference/types/struct.html), [Enums](https://doc.rust-lang.org/reference/types/enum.html)
///   and [Unions](https://doc.rust-lang.org/reference/types/union.html)
#[repr(C)]
#[derive(Debug)]
pub struct SynPathTy<'ast> {
    data: CommonSynTyData<'ast>,
    path: AstQPath<'ast>,
}

impl<'ast> SynPathTy<'ast> {
    pub fn path(&self) -> &AstQPath<'ast> {
        &self.path
    }
}

super::impl_ty_data!(SynPathTy<'ast>, Path);

#[cfg(feature = "driver-api")]
impl<'ast> SynPathTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, path: AstQPath<'ast>) -> Self {
        Self { data, path }
    }
}

/// The semantic representation of an abstract data type. This can be an
/// [`Enum`], [`Struct`], or [`Union`].
///
/// [`Struct`]: https://doc.rust-lang.org/reference/types/struct.html
/// [`Enum`]: https://doc.rust-lang.org/reference/types/enum.html
/// [`Union`]: https://doc.rust-lang.org/reference/types/union.html
#[repr(C)]
#[derive(Debug)]
pub struct SemAdtTy<'ast> {
    def_id: TyDefId,
    generics: SemGenericArgs<'ast>,
}

impl<'ast> SemAdtTy<'ast> {
    /// This returns the [`TyDefId`] of the abstract data type.
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

    /// This returns the [`SemGenericArgs`] used by the type
    pub fn generics(&self) -> &SemGenericArgs<'ast> {
        &self.generics
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemAdtTy<'ast> {
    pub fn new(def_id: TyDefId, generics: SemGenericArgs<'ast>) -> Self {
        Self { def_id, generics }
    }
}

/// The semantic representation of a generic type. For example
///
/// ```
/// fn function<T: Default>() {
///     let _ = T::default();
///     //      ^^^^^^^^^^^^ This will have the generic type `T`
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct SemGenericTy<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    generic_id: GenericId,
}

impl<'ast> SemGenericTy<'ast> {
    /// This returns the [`GenericId`] assigned to the generic parameter.
    /// This id can be used to retrieve more information from the item that
    /// defines the generic.
    pub fn generic_id(&self) -> GenericId {
        self.generic_id
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemGenericTy<'ast> {
    pub fn new(generic_id: GenericId) -> Self {
        Self {
            _lifetime: PhantomData,
            generic_id,
        }
    }
}

/// The semantic representation of a type alias.
///
/// Aliases in semantic type representations are usually resolved directly. This
/// kind, is primarily used for instances, where the concrete aliased type is not yet
/// known.
#[repr(C)]
#[derive(Debug)]
pub struct SemAliasTy<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    alias_item: ItemId,
}

impl<'ast> SemAliasTy<'ast> {
    /// This [`ItemId`] identifies the item that defined the alias
    pub fn alias_item(&self) -> ItemId {
        self.alias_item
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemAliasTy<'ast> {
    pub fn new(alias_item: ItemId) -> Self {
        Self {
            _lifetime: PhantomData,
            alias_item,
        }
    }
}
