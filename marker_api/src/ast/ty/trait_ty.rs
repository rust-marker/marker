use crate::{ast::generic::TyParamBound, ffi::FfiSlice};

use super::CommonSynTyData;

#[repr(C)]
#[derive(Debug)]
pub struct ImplTraitTy<'ast> {
    data: CommonSynTyData<'ast>,
    trait_bound: FfiSlice<'ast, TyParamBound<'ast>>,
}

super::impl_ty_data!(ImplTraitTy<'ast>, ImplTrait);

impl<'ast> ImplTraitTy<'ast> {
    pub fn trait_bounds(&self) -> &[TyParamBound<'ast>] {
        self.trait_bound.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ImplTraitTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, trait_bound: &'ast [TyParamBound<'ast>]) -> Self {
        Self {
            data,
            trait_bound: trait_bound.into(),
        }
    }
}

/// The syntactic representation of a [trait object].
///
/// [trait object]: https://doc.rust-lang.org/reference/types/trait-object.html
#[repr(C)]
#[derive(Debug)]
pub struct TraitObjTy<'ast> {
    data: CommonSynTyData<'ast>,
    trait_bound: FfiSlice<'ast, TyParamBound<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitObjTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, trait_bound: &'ast [TyParamBound<'ast>]) -> Self {
        Self {
            data,
            trait_bound: trait_bound.into(),
        }
    }
}

super::impl_ty_data!(TraitObjTy<'ast>, TraitObj);

impl<'ast> TraitObjTy<'ast> {
    pub fn trait_bounds(&self) -> &[TyParamBound<'ast>] {
        self.trait_bound.get()
    }
}
