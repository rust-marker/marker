use crate::{ffi::FfiSlice, sem::generic::SemTraitBound};

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
