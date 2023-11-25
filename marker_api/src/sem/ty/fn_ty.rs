use crate::{
    common::{ItemId, TyDefId},
    sem::generic::GenericArgs,
};

use super::CommonTyData;

/// A [function item type](https://doc.rust-lang.org/reference/types/function-item.html)
/// identifying a specific function and potentualy additional generics.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct FnTy<'ast> {
    data: CommonTyData<'ast>,
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

super::impl_ty_data!(FnTy<'ast>, Fn);

/// The semantic representation of a
/// [closure type](https://doc.rust-lang.org/reference/types/closure.html).
///
/// Closure expressions create anonymous types, which implement traits to call the
/// closure. This type on it's own therefore only identifies the type of the closure.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct ClosureTy<'ast> {
    data: CommonTyData<'ast>,
    ty_id: TyDefId,
    generics: GenericArgs<'ast>,
}

impl<'ast> ClosureTy<'ast> {
    /// This returns the [`ItemId`] of the struct that was generated for this closure.
    pub fn closure_ty_id(&self) -> TyDefId {
        self.ty_id
    }

    /// This returns the [`GenericArgs`] used by closure.
    pub fn generics(&self) -> &GenericArgs<'ast> {
        &self.generics
    }

    // FIXME: Add a method to get a different representation, which includes the
    // parameters and return type.
}

super::impl_ty_data!(ClosureTy<'ast>, Closure);
