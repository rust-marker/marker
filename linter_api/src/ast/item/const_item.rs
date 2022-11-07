use crate::ast::{ty::TyKind, BodyId};

use super::CommonItemData;

/// A module item like:
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
    ty: TyKind<'ast>,
    body_id: Option<BodyId>,
}

impl<'ast> ConstItem<'ast> {
    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    pub fn body_id(&self) -> Option<BodyId> {
        self.body_id
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, ty: TyKind<'ast>, body_id: Option<BodyId>) -> Self {
        Self { data, ty, body_id }
    }
}

super::impl_item_data!(ConstItem, Const);
