use crate::ast::ItemId;

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SelfTy<'ast> {
    data: CommonTyData<'ast>,
    item: ItemId,
}

super::impl_ty_data!(SelfTy<'ast>, SelfTy);

impl<'ast> SelfTy<'ast> {
    /// This returns the [`ItemId`] that this `Self` originates from.
    pub fn self_source_item_id(&self) -> ItemId {
        self.item
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SelfTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, item: ItemId) -> Self {
        Self { data, item }
    }
}
