use std::marker::PhantomData;

use crate::{
    common::{GenericId, ItemId, TyDefId},
    sem::generic::GenericArgs,
};

/// The semantic representation of an abstract data type. This can be an
/// [`Enum`], [`Struct`], or [`Union`].
///
/// [`Struct`]: https://doc.rust-lang.org/reference/types/struct.html
/// [`Enum`]: https://doc.rust-lang.org/reference/types/enum.html
/// [`Union`]: https://doc.rust-lang.org/reference/types/union.html
#[repr(C)]
#[derive(Debug)]
pub struct AdtTy<'ast> {
    def_id: TyDefId,
    generics: GenericArgs<'ast>,
}

impl<'ast> AdtTy<'ast> {
    /// This returns the [`TyDefId`] of the abstract data type.
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

    /// This returns the [`GenericArgs`] used by the type
    pub fn generics(&self) -> &GenericArgs<'ast> {
        &self.generics
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> AdtTy<'ast> {
    pub fn new(def_id: TyDefId, generics: GenericArgs<'ast>) -> Self {
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
pub struct GenericTy<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    generic_id: GenericId,
}

impl<'ast> GenericTy<'ast> {
    /// This returns the [`GenericId`] assigned to the generic parameter.
    /// This id can be used to retrieve more information from the item that
    /// defines the generic.
    pub fn generic_id(&self) -> GenericId {
        self.generic_id
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> GenericTy<'ast> {
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
pub struct AliasTy<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    alias_item: ItemId,
}

impl<'ast> AliasTy<'ast> {
    /// This [`ItemId`] identifies the item that defined the alias
    pub fn alias_item(&self) -> ItemId {
        self.alias_item
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> AliasTy<'ast> {
    pub fn new(alias_item: ItemId) -> Self {
        Self {
            _lifetime: PhantomData,
            alias_item,
        }
    }
}
