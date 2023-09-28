use crate::ast::{ty::SynTyKind, BodyId};
use crate::ffi::FfiOption;

use super::CommonItemData;

/// A const item like:
///
/// ```
/// const CONST_ITEM: u32 = 0xcafe;
/// ```
///
/// * See <https://doc.rust-lang.org/stable/reference/items/constant-items.html>
#[repr(C)]
#[derive(Debug)]
pub struct ConstItem<'ast> {
    data: CommonItemData<'ast>,
    ty: SynTyKind<'ast>,
    body_id: FfiOption<BodyId>,
}

impl<'ast> ConstItem<'ast> {
    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    pub fn body_id(&self) -> Option<BodyId> {
        self.body_id.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, ty: SynTyKind<'ast>, body_id: Option<BodyId>) -> Self {
        Self {
            data,
            ty,
            body_id: body_id.into(),
        }
    }
}

super::impl_item_data!(ConstItem, Const);
