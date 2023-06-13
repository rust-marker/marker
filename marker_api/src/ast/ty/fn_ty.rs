use std::marker::PhantomData;

use crate::ast::{generic::SemGenericArgs, ItemId};

/// A [function item type](https://doc.rust-lang.org/reference/types/function-item.html)
/// identifying a specific function and potentualy additional generics.
#[repr(C)]
#[derive(Debug)]
pub struct SemFnTy<'ast> {
    fn_id: ItemId,
    generics: SemGenericArgs<'ast>,
}

impl<'ast> SemFnTy<'ast> {
    /// This returns the [`ItemId`] of the identified function.
    pub fn fn_id(&self) -> ItemId {
        self.fn_id
    }

    /// This returns the [`SemGenericArgs`] used by identified function
    pub fn generics(&self) -> &SemGenericArgs<'ast> {
        &self.generics
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemFnTy<'ast> {
    pub fn new(fn_id: ItemId, generics: SemGenericArgs<'ast>) -> Self {
        Self { fn_id, generics }
    }
}
