use crate::{ffi::FfiSlice, sem::generic::TraitBound};

/// The semantic representation of a [trait object].
///
/// [trait object]: https://doc.rust-lang.org/reference/types/trait-object.html
#[repr(C)]
#[derive(Debug)]
pub struct TraitObjTy<'ast> {
    bound: FfiSlice<'ast, TraitBound<'ast>>,
}

impl<'ast> TraitObjTy<'ast> {
    pub fn bounds(&self) -> &[TraitBound<'ast>] {
        self.bound.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitObjTy<'ast> {
    pub fn new(bound: &'ast [TraitBound<'ast>]) -> Self {
        Self { bound: bound.into() }
    }
}
