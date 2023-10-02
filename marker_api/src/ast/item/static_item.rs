use crate::{
    ast::{ty::SynTyKind, Mutability},
    common::BodyId,
    ffi::FfiOption,
};

use super::CommonItemData;

/// A static item like:
///
/// ```
/// static mut LEVELS: u32 = 0;
/// // `name()` -> `LEVELS`
/// // `mutability()` -> `Mutability::Mut`
/// // `ty()` -> _Ty of u32_
/// // `body_id()` -> _BodyId of `0`_
/// ```
///
/// See: <https://doc.rust-lang.org/stable/reference/items/static-items.html>
#[repr(C)]
#[derive(Debug)]
pub struct StaticItem<'ast> {
    data: CommonItemData<'ast>,
    mutability: Mutability,
    body_id: FfiOption<BodyId>,
    ty: SynTyKind<'ast>,
}

super::impl_item_data!(StaticItem, Static);

impl<'ast> StaticItem<'ast> {
    /// The mutability of this item
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// The defined type of this static item
    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    /// This returns the [`BodyId`] of the initialization body.
    pub fn body_id(&self) -> Option<BodyId> {
        self.body_id.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> StaticItem<'ast> {
    pub fn new(
        data: CommonItemData<'ast>,
        mutability: Mutability,
        body_id: Option<BodyId>,
        ty: SynTyKind<'ast>,
    ) -> Self {
        Self {
            data,
            mutability,
            body_id: body_id.into(),
            ty,
        }
    }
}
