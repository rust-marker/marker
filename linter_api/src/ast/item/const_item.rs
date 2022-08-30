use crate::ast::BodyId;

use super::CommonItemData;

/// A module item like:
///
/// ```
/// const CONST_ITEM: u32 = 0xcafe;
/// ```
///
/// * See <https://doc.rust-lang.org/stable/reference/items/constant-items.html>
#[derive(Debug)]
pub struct ConstItem<'ast> {
    data: CommonItemData<'ast>,
    body_id: Option<BodyId>,
}

impl<'ast> ConstItem<'ast> {
    pub fn ty(&self) {
        todo!();
    }

    pub fn body_id(&self) -> Option<BodyId> {
        self.body_id
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, body_id: Option<BodyId>) -> Self {
        Self { data, body_id }
    }
}

super::impl_item_data!(ConstItem, Const);
