use std::marker::PhantomData;

use crate::ast::{generic::SemGenericArgs, GenericId, ItemId, TyDefId};

#[repr(C)]
#[derive(Debug)]
pub struct SemAdtTy<'ast> {
    def_id: TyDefId,
    generics: SemGenericArgs<'ast>,
}

impl<'ast> SemAdtTy<'ast> {
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

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

#[repr(C)]
#[derive(Debug)]
pub struct SemGenericTy<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    generic_id: GenericId,
}

impl<'ast> SemGenericTy<'ast> {
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

#[repr(C)]
#[derive(Debug)]
pub struct SemAliasTy<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    alias_item: ItemId,
}

impl<'ast> SemAliasTy<'ast> {
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
