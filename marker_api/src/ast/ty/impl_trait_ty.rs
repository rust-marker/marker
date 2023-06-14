use crate::{ast::generic::TyParamBound, ffi::FfiSlice};

use super::CommonSynTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ImplTraitTy<'ast> {
    data: CommonSynTyData<'ast>,
    trait_bound: FfiSlice<'ast, TyParamBound<'ast>>,
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

super::impl_ty_data!(ImplTraitTy<'ast>, ImplTrait);

impl<'ast> ImplTraitTy<'ast> {
    pub fn trait_bounds(&self) -> &[TyParamBound<'ast>] {
        self.trait_bound.get()
    }
}
