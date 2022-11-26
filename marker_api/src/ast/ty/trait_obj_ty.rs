use crate::{ast::generic::TyParamBound, ffi::FfiSlice};

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TraitObjTy<'ast> {
    data: CommonTyData<'ast>,
    trait_bound: FfiSlice<'ast, TyParamBound<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitObjTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, trait_bound: &'ast [TyParamBound<'ast>]) -> Self {
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
