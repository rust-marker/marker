use crate::ast::{generic::SemGenericArgs, ItemId, TyDefId};

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

/// The semantic representation of a
/// [closure type](https://doc.rust-lang.org/reference/types/closure.html).
///
/// Closure expressions create anonymous types, which implement traits to call the
/// closure. This type on it's own therefore only identifies the type of the closure.
#[repr(C)]
#[derive(Debug)]
pub struct SemClosureTy<'ast> {
    closure_ty_id: TyDefId,
    generics: SemGenericArgs<'ast>,
}

impl<'ast> SemClosureTy<'ast> {
    /// This returns the [`ItemId`] of the identified function.
    pub fn closure_ty_id(&self) -> TyDefId {
        self.closure_ty_id
    }

    /// This returns the [`SemGenericArgs`] used by identified function
    pub fn generics(&self) -> &SemGenericArgs<'ast> {
        &self.generics
    }

    // FIXME: Add a method to get a different representation, which includes the
    // parameters and return type.
}

#[cfg(feature = "driver-api")]
impl<'ast> SemClosureTy<'ast> {
    pub fn new(closure_ty_id: TyDefId, generics: SemGenericArgs<'ast>) -> Self {
        Self {
            closure_ty_id,
            generics,
        }
    }
}
