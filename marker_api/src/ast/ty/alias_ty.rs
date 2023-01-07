use crate::ast::ItemId;

use super::CommonTyData;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AliasTy<'ast> {
    data: CommonTyData<'ast>,
    item: ItemId,
}

super::impl_ty_data!(AliasTy<'ast>, Alias);

impl<'ast> AliasTy<'ast> {
    /// This returns the [`ItemId`] belonging to the
    /// [`TyAliasItem`](`super::super::item::TyAliasItem`) that declared this item.
    pub fn alias_item_id(&self) -> ItemId {
        self.item
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> AliasTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, item: ItemId) -> Self {
        Self { data, item }
    }
}
