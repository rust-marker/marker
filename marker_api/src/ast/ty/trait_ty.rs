use crate::{ast::generic::SemTyParamBound, ffi::FfiSlice};

#[repr(C)]
#[derive(Debug)]
pub struct SemTraitObjTy<'ast> {
    bound: FfiSlice<'ast, SemTyParamBound<'ast>>,
}

impl<'ast> SemTraitObjTy<'ast> {
    pub fn bounds(&self) -> &[SemTyParamBound<'ast>] {
        self.bound.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTraitObjTy<'ast> {
    pub fn new(bound: &'ast [SemTyParamBound<'ast>]) -> Self {
        Self { bound: bound.into() }
    }
}
