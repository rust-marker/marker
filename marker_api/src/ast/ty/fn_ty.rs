use crate::ast::{generic::SemGenericArgs, impl_callable_data_trait, CommonCallableData, ItemId, TyDefId};

use super::CommonSynTyData;

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

/// The syntactic representation of a
/// [closure type](https://doc.rust-lang.org/reference/types/closure.html).
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SynClosureTy<'ast> {
    data: CommonSynTyData<'ast>,
    callable_data: CommonCallableData<'ast>,
    // FIXME: Add support for `for<'lifetime>` binder
    // FIXME: Potentially add functions to check which [`Fn`] traits this implements
}

#[cfg(feature = "driver-api")]
impl<'ast> SynClosureTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, callable_data: CommonCallableData<'ast>) -> Self {
        Self { data, callable_data }
    }
}

super::impl_ty_data!(SynClosureTy<'ast>, Closure);
impl_callable_data_trait!(SynClosureTy<'ast>);

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
