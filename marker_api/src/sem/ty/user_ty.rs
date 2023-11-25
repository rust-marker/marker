use crate::{
    common::{GenericId, ItemId, TyDefId},
    sem::generic::GenericArgs,
};

use super::CommonTyData;

/// The semantic representation of an abstract data type. This can be an
/// [`Enum`], [`Struct`], or [`Union`].
///
/// [`Struct`]: https://doc.rust-lang.org/reference/types/struct.html
/// [`Enum`]: https://doc.rust-lang.org/reference/types/enum.html
/// [`Union`]: https://doc.rust-lang.org/reference/types/union.html
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct AdtTy<'ast> {
    data: CommonTyData<'ast>,
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

super::impl_ty_data!(AdtTy<'ast>, Adt);

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
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct GenericTy<'ast> {
    data: CommonTyData<'ast>,
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

super::impl_ty_data!(GenericTy<'ast>, Generic);

/// The semantic representation of a type alias.
///
/// Aliases in semantic type representations are usually resolved directly. This
/// kind, is primarily used for instances, where the concrete aliased type is not yet
/// known.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct AliasTy<'ast> {
    data: CommonTyData<'ast>,
    alias_item: ItemId,
}

impl<'ast> AliasTy<'ast> {
    /// This [`ItemId`] identifies the item that defined the alias
    pub fn alias_item(&self) -> ItemId {
        self.alias_item
    }
}

super::impl_ty_data!(AliasTy<'ast>, Alias);
