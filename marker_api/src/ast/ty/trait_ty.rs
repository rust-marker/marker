use crate::{
    ast::generic::{SemTraitBound, TyParamBound},
    ffi::FfiSlice,
};

use super::CommonSynTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SynImplTraitTy<'ast> {
    data: CommonSynTyData<'ast>,
    trait_bound: FfiSlice<'ast, TyParamBound<'ast>>,
}

super::impl_ty_data!(SynImplTraitTy<'ast>, ImplTrait);

impl<'ast> SynImplTraitTy<'ast> {
    pub fn trait_bounds(&self) -> &[TyParamBound<'ast>] {
        self.trait_bound.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynImplTraitTy<'ast> {
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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SynTraitObjTy<'ast> {
    data: CommonSynTyData<'ast>,
    trait_bound: FfiSlice<'ast, TyParamBound<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SynTraitObjTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, trait_bound: &'ast [TyParamBound<'ast>]) -> Self {
        Self {
            data,
            trait_bound: trait_bound.into(),
        }
    }
}

super::impl_ty_data!(SynTraitObjTy<'ast>, TraitObj);

impl<'ast> SynTraitObjTy<'ast> {
    pub fn trait_bounds(&self) -> &[TyParamBound<'ast>] {
        self.trait_bound.get()
    }
}

/// The semantic representation of a [trait object].
///
/// [trait object]: https://doc.rust-lang.org/reference/types/trait-object.html
#[repr(C)]
#[derive(Debug)]
pub struct SemTraitObjTy<'ast> {
    bound: FfiSlice<'ast, SemTraitBound<'ast>>,
}

impl<'ast> SemTraitObjTy<'ast> {
    pub fn bounds(&self) -> &[SemTraitBound<'ast>] {
        self.bound.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTraitObjTy<'ast> {
    pub fn new(bound: &'ast [SemTraitBound<'ast>]) -> Self {
        Self { bound: bound.into() }
    }
}
