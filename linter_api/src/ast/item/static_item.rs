use crate::ast::{
    ty::{Mutability, Ty},
    BodyId,
};

use super::{ItemBase, ItemBaseData, ItemType};

/// ```ignore
/// static mut LEVELS: u32 = 0;
/// // `get_name()` -> `LEVELS`
/// // `get_mutability()` -> _Mutable_
/// // `get_ty()` -> _Ty of u32_
/// // `get_body_id()` -> _BodyId of `0`_
/// ```
pub type StaticItem<'ast> = ItemBase<'ast, StaticItemData>;

#[derive(Debug)]
pub struct StaticItemData {
    mutability: Mutability,
    body_id: BodyId,
}

impl<'ast> ItemBaseData<'ast> for StaticItemData {
    fn as_item_type(base: &'ast ItemBase<'ast, Self>) -> super::ItemType<'ast> {
        ItemType::Static(base)
    }
}

impl<'ast> StaticItem<'ast> {
    /// The mutability of this item
    pub fn get_mutability(&self) -> Mutability {
        self.data.mutability
    }

    /// The defined type of this static item
    pub fn get_ty(&'ast self) -> &'ast dyn Ty<'ast> {
        todo!()
    }

    /// This returns the [`BodyId`] of the initialization body.
    pub fn get_body_id(&self) -> BodyId {
        self.data.body_id
    }
}
