use crate::{ffi::FfiSlice, sem::generic::TraitBound};

use super::CommonTyData;

/// The semantic representation of a [trait object].
///
/// [trait object]: https://doc.rust-lang.org/reference/types/trait-object.html
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct TraitObjTy<'ast> {
    data: CommonTyData<'ast>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    bounds: FfiSlice<'ast, TraitBound<'ast>>,
}

impl<'ast> TraitObjTy<'ast> {
    pub fn bounds(&self) -> &[TraitBound<'ast>] {
        self.bounds.get()
    }
}

super::impl_ty_data!(TraitObjTy<'ast>, TraitObj);
