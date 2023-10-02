use crate::{
    common::{ItemId, TyDefId},
    sem::generic::GenericArgs,
};

/// A [function item type](https://doc.rust-lang.org/reference/types/function-item.html)
/// identifying a specific function and potentualy additional generics.
#[repr(C)]
#[derive(Debug)]
pub struct FnTy<'ast> {
    fn_id: ItemId,
    generics: GenericArgs<'ast>,
}

impl<'ast> FnTy<'ast> {
    /// This returns the [`ItemId`] of the identified function.
    pub fn fn_id(&self) -> ItemId {
        self.fn_id
    }

    /// This returns the [`GenericArgs`] used by identified function
    pub fn generics(&self) -> &GenericArgs<'ast> {
        &self.generics
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> FnTy<'ast> {
    pub fn new(fn_id: ItemId, generics: GenericArgs<'ast>) -> Self {
        Self { fn_id, generics }
    }
}

/// The semantic representation of a
/// [closure type](https://doc.rust-lang.org/reference/types/closure.html).
///
/// Closure expressions create anonymous types, which implement traits to call the
/// closure. This type on it's own therefore only identifies the type of the closure.
#[repr(C)]
#[derive(Debug)]
pub struct ClosureTy<'ast> {
    closure_ty_id: TyDefId,
    generics: GenericArgs<'ast>,
}

impl<'ast> ClosureTy<'ast> {
    /// This returns the [`ItemId`] of the identified function.
    pub fn closure_ty_id(&self) -> TyDefId {
        self.closure_ty_id
    }

    /// This returns the [`GenericArgs`] used by identified function
    pub fn generics(&self) -> &GenericArgs<'ast> {
        &self.generics
    }

    // FIXME: Add a method to get a different representation, which includes the
    // parameters and return type.
}

#[cfg(feature = "driver-api")]
impl<'ast> ClosureTy<'ast> {
    pub fn new(closure_ty_id: TyDefId, generics: GenericArgs<'ast>) -> Self {
        Self {
            closure_ty_id,
            generics,
        }
    }
}
